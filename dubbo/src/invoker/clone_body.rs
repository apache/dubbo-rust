use std::{
    collections::VecDeque,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures_core::ready;

use http::HeaderMap;
use http_body::Body;
use pin_project::pin_project;
use thiserror::Error;

use crate::StdError;

#[derive(Error, Debug)]
#[error("buffered body reach max capacity.")]
pub struct ReachMaxCapacityError;

pub struct BufferedBody {
    shared: Arc<Mutex<Option<OwnedBufferedBody>>>,
    owned: Option<OwnedBufferedBody>,
    replay_body: bool,
    replay_trailers: bool,
    is_empty: bool,
    size_hint: http_body::SizeHint,
}

pub struct OwnedBufferedBody {
    body: hyper::Body,
    trailers: Option<HeaderMap>,
    buf: InnerBuffer,
}

impl BufferedBody {
    pub fn new(body: hyper::Body, buf_size: usize) -> Self {
        let size_hint = body.size_hint();
        let is_empty = body.is_end_stream();
        BufferedBody {
            shared: Default::default(),
            owned: Some(OwnedBufferedBody {
                body,
                trailers: None,
                buf: InnerBuffer {
                    bufs: Default::default(),
                    capacity: buf_size,
                },
            }),
            replay_body: false,
            replay_trailers: false,
            is_empty,
            size_hint,
        }
    }
}

impl Clone for BufferedBody {
    fn clone(&self) -> Self {
        Self {
            shared: self.shared.clone(),
            owned: None,
            replay_body: true,
            replay_trailers: true,
            is_empty: self.is_empty,
            size_hint: self.size_hint.clone(),
        }
    }
}

impl Drop for BufferedBody {
    fn drop(&mut self) {
        if let Some(owned) = self.owned.take() {
            let lock = self.shared.lock();
            if let Ok(mut lock) = lock {
                *lock = Some(owned);
            }
        }
    }
}

impl Body for BufferedBody {
    type Data = BytesData;
    type Error = StdError;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let mut_self = self.get_mut();

        let owned_body = mut_self.owned.get_or_insert_with(|| {
            let lock = mut_self.shared.lock();
            if let Err(e) = lock {
                panic!("buffered body get shared data lock failed. {}", e);
            }
            let mut data = lock.unwrap();

            data.take().expect("cannot get shared buffered body.")
        });

        if mut_self.replay_body {
            mut_self.replay_body = false;
            if owned_body.buf.has_remaining() {
                return Poll::Ready(Some(Ok(BytesData::BufferedBytes(owned_body.buf.clone()))));
            }

            if owned_body.buf.is_capped() {
                return Poll::Ready(Some(Err(ReachMaxCapacityError.into())));
            }
        }

        if mut_self.is_empty {
            return Poll::Ready(None);
        }

        let mut data = {
            let pin = Pin::new(&mut owned_body.body);
            let data = ready!(pin.poll_data(cx));
            match data {
                Some(Ok(data)) => data,
                Some(Err(e)) => return Poll::Ready(Some(Err(e.into()))),
                None => {
                    mut_self.is_empty = true;
                    return Poll::Ready(None);
                }
            }
        };

        let len = data.remaining();

        owned_body.buf.capacity = owned_body.buf.capacity.saturating_sub(len);

        let data = if owned_body.buf.is_capped() {
            if owned_body.buf.has_remaining() {
                owned_body.buf.bufs = VecDeque::default();
            }
            data.copy_to_bytes(len)
        } else {
            owned_body.buf.push_bytes(data.copy_to_bytes(len))
        };

        Poll::Ready(Some(Ok(BytesData::OriginBytes(data))))
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<http::HeaderMap>, Self::Error>> {
        let mut_self = self.get_mut();
        let owned_body = mut_self.owned.get_or_insert_with(|| {
            let lock = mut_self.shared.lock();
            if let Err(e) = lock {
                panic!("buffered body get shared data lock failed. {}", e);
            }
            let mut data = lock.unwrap();

            data.take().expect("cannot get shared buffered body.")
        });

        if mut_self.replay_trailers {
            mut_self.replay_trailers = false;
            if let Some(ref trailers) = owned_body.trailers {
                return Poll::Ready(Ok(Some(trailers.clone())));
            }
        }

        let mut_body = &mut owned_body.body;
        if !mut_body.is_end_stream() {
            let trailers = ready!(Pin::new(mut_body).poll_trailers(cx)).map(|trailers| {
                owned_body.trailers = trailers.clone();
                trailers
            });
            return Poll::Ready(trailers.map_err(|e| e.into()));
        }

        Poll::Ready(Ok(None))
    }

    fn is_end_stream(&self) -> bool {
        if self.is_empty {
            return true;
        }

        let is_end = self
            .owned
            .as_ref()
            .map(|owned| owned.body.is_end_stream())
            .unwrap_or(false);

        !self.replay_body && !self.replay_trailers && is_end
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.size_hint.clone()
    }
}

#[derive(Clone)]
pub struct InnerBuffer {
    bufs: VecDeque<Bytes>,
    capacity: usize,
}

impl InnerBuffer {
    pub fn push_bytes(&mut self, bytes: Bytes) -> Bytes {
        self.bufs.push_back(bytes.clone());
        bytes
    }

    pub fn is_capped(&self) -> bool {
        self.capacity == 0
    }
}

impl Buf for InnerBuffer {
    fn remaining(&self) -> usize {
        self.bufs.iter().map(|bytes| bytes.remaining()).sum()
    }

    fn chunk(&self) -> &[u8] {
        self.bufs.front().map(Buf::chunk).unwrap_or(&[])
    }

    fn chunks_vectored<'a>(&'a self, dst: &mut [std::io::IoSlice<'a>]) -> usize {
        if dst.is_empty() {
            return 0;
        }

        let mut filled = 0;

        for bytes in self.bufs.iter() {
            filled += bytes.chunks_vectored(&mut dst[filled..])
        }

        filled
    }

    fn advance(&mut self, mut cnt: usize) {
        while cnt > 0 {
            let first = self.bufs.front_mut();
            if first.is_none() {
                break;
            }
            let first = first.unwrap();
            let first_remaining = first.remaining();
            if first_remaining > cnt {
                first.advance(cnt);
                break;
            }

            first.advance(first_remaining);
            cnt = cnt - first_remaining;
            self.bufs.pop_front();
        }
    }

    fn copy_to_bytes(&mut self, len: usize) -> bytes::Bytes {
        match self.bufs.front_mut() {
            Some(buf) if len <= buf.remaining() => {
                let bytes = buf.copy_to_bytes(len);
                if buf.remaining() == 0 {
                    self.bufs.pop_front();
                }
                bytes
            }
            _ => {
                let mut bytes = BytesMut::with_capacity(len);
                bytes.put(self.take(len));
                bytes.freeze()
            }
        }
    }
}

pub enum BytesData {
    BufferedBytes(InnerBuffer),
    OriginBytes(Bytes),
}

impl Buf for BytesData {
    fn remaining(&self) -> usize {
        match self {
            BytesData::BufferedBytes(bytes) => bytes.remaining(),
            BytesData::OriginBytes(bytes) => bytes.remaining(),
        }
    }

    fn chunk(&self) -> &[u8] {
        match self {
            BytesData::BufferedBytes(bytes) => bytes.chunk(),
            BytesData::OriginBytes(bytes) => bytes.chunk(),
        }
    }

    fn advance(&mut self, cnt: usize) {
        match self {
            BytesData::BufferedBytes(bytes) => bytes.advance(cnt),
            BytesData::OriginBytes(bytes) => bytes.advance(cnt),
        }
    }

    fn copy_to_bytes(&mut self, len: usize) -> bytes::Bytes {
        match self {
            BytesData::BufferedBytes(bytes) => bytes.copy_to_bytes(len),
            BytesData::OriginBytes(bytes) => bytes.copy_to_bytes(len),
        }
    }
}

#[pin_project]
pub struct CloneBody(#[pin] BufferedBody);

impl CloneBody {
    pub fn new(inner_body: hyper::Body) -> Self {
        let inner_body = BufferedBody::new(inner_body, 1024 * 64);
        CloneBody(inner_body)
    }
}

impl Body for CloneBody {
    type Data = BytesData;

    type Error = StdError;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        self.project().0.poll_data(cx)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<http::HeaderMap>, Self::Error>> {
        self.project().0.poll_trailers(cx)
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.0.size_hint()
    }
}

impl Clone for CloneBody {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

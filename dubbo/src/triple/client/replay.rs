use std::{
    collections::VecDeque,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures_core::{ready, stream::BoxStream, Stream};
use futures_util::StreamExt;
use http_body::Body;
use pin_project::pin_project;

use crate::status::Status;

type BoxBytesStream = BoxStream<'static, Result<Bytes, Status>>;
pub struct ClonedBytesStream {
    shared: Arc<Mutex<Option<OwnedBytesStream>>>,
    owned: Option<OwnedBytesStream>,
    replay: bool,
}

pub struct OwnedBytesStream {
    bytes_stream: BoxBytesStream,
    buf: InnerBuffer,
}

#[derive(Clone)]
pub struct InnerBuffer {
    bufs: VecDeque<Bytes>,
    capacity: usize,
}

impl ClonedBytesStream {
    pub fn new(buffer_capacity: usize, stream: BoxBytesStream) -> Self {
        Self {
            shared: Arc::new(Mutex::new(None)),
            owned: Some(OwnedBytesStream {
                bytes_stream: stream,
                buf: InnerBuffer {
                    bufs: Default::default(),
                    capacity: buffer_capacity,
                },
            }),
            replay: false,
        }
    }
}

impl Clone for ClonedBytesStream {
    fn clone(&self) -> Self {
        Self {
            shared: self.shared.clone(),
            owned: None,
            replay: true,
        }
    }
}

impl Drop for ClonedBytesStream {
    fn drop(&mut self) {
        if let Some(owned) = self.owned.take() {
            let lock = self.shared.lock();
            if let Ok(mut lock) = lock {
                *lock = Some(owned);
            }
        }
    }
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

impl Stream for ClonedBytesStream {
    type Item = Result<BytesData, Status>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut_self = self.get_mut();

        let owned_bytes_stream = mut_self.owned.get_or_insert_with(|| {
            let lock = mut_self.shared.lock();
            if let Err(e) = lock {
                panic!("bytes streams get shared data lock failed. {}", e);
            }
            let mut data = lock.unwrap();

            data.take().expect("cannot get shared bytes streams.")
        });

        if mut_self.replay {
            mut_self.replay = false;
            if owned_bytes_stream.buf.has_remaining() {
                return Poll::Ready(Some(Ok(BytesData::BufferedBytes(
                    owned_bytes_stream.buf.clone(),
                ))));
            }
        }

        let next = owned_bytes_stream.bytes_stream.poll_next_unpin(cx);

        let next = match ready!(next) {
            Some(next) => match next {
                Ok(next) => next,
                Err(e) => return Poll::Ready(Some(Err(e))),
            },
            _ => return Poll::Ready(None),
        };

        let len = next.len();
        owned_bytes_stream.buf.capacity = owned_bytes_stream.buf.capacity.saturating_sub(len);
        let next = if owned_bytes_stream.buf.is_capped() {
            if owned_bytes_stream.buf.has_remaining() {
                owned_bytes_stream.buf.bufs = VecDeque::default();
            }
            next
        } else {
            owned_bytes_stream.buf.push_bytes(next)
        };

        return Poll::Ready(Some(Ok(BytesData::OriginBytes(next))));
    }
}

#[pin_project]
#[derive(Clone)]
pub struct ClonedBody(#[pin] ClonedBytesStream);

impl ClonedBody {
    pub fn new<T>(inner_body: T) -> Self
    where
        T: Stream<Item = Result<Bytes, Status>> + Send + 'static,
    {
        let inner_body = Box::pin(inner_body);
        let inner_body = ClonedBytesStream::new(1024 * 64, inner_body);
        ClonedBody(inner_body)
    }
}

impl Body for ClonedBody {
    type Data = BytesData;

    type Error = Status;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        self.project().0.poll_next(cx)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<http::HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }

    fn size_hint(&self) -> http_body::SizeHint {
        http_body::SizeHint::default()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cloned_bytes_stream() {
        let bytes1 = Ok(Bytes::from("hello".as_bytes()));
        let bytes2 = Ok(Bytes::from(" world".as_bytes()));
        let bytes3 = Ok(Bytes::from(", end test!".as_bytes()));

        let stream = futures_util::stream::iter(vec![bytes1, bytes2, bytes3]);

        let mut origin_stream = ClonedBytesStream::new(64 * 1024, Box::pin(stream));
        let hello_bytes = origin_stream.next().await;

        assert!(hello_bytes.is_some());

        let hello_bytes = hello_bytes.unwrap();

        assert!(hello_bytes.is_ok());

        assert_eq!(bytes_data_to_str(hello_bytes.unwrap()), "hello");

        let world_bytes = origin_stream.next().await;

        assert!(world_bytes.is_some());

        let world_bytes = world_bytes.unwrap();

        assert!(world_bytes.is_ok());

        assert_eq!(bytes_data_to_str(world_bytes.unwrap()), " world");

        let end_bytes = origin_stream.next().await;

        assert!(end_bytes.is_some());

        let end_bytes = end_bytes.unwrap();

        assert!(end_bytes.is_ok());

        assert_eq!(bytes_data_to_str(end_bytes.unwrap()), ", end test!");

        let none_bytes = origin_stream.next().await;

        assert!(none_bytes.is_none());
    }

    #[tokio::test]
    async fn test_cloned_bytes_stream_and_replay() {
        let bytes1 = Ok(Bytes::from("hello".as_bytes()));
        let bytes2 = Ok(Bytes::from(" world".as_bytes()));
        let bytes3 = Ok(Bytes::from(", end test!".as_bytes()));

        let stream = futures_util::stream::iter(vec![bytes1, bytes2, bytes3]);

        let mut origin_stream = ClonedBytesStream::new(64 * 1024, Box::pin(stream));
        origin_stream.next().await;
        origin_stream.next().await;
        origin_stream.next().await;

        let none_bytes = origin_stream.next().await;
        assert!(none_bytes.is_none());

        let mut clone_origin_stream = origin_stream.clone();

        drop(origin_stream);

        let cached_bytes = clone_origin_stream.next().await;
        assert!(cached_bytes.is_some());

        let cached_bytes = cached_bytes.unwrap();

        assert!(cached_bytes.is_ok());

        assert_eq!(
            bytes_data_to_str(cached_bytes.unwrap()),
            "hello world, end test!"
        );

        let none_bytes = clone_origin_stream.next().await;
        assert!(none_bytes.is_none());
    }

    #[tokio::test]
    async fn test_replay_stream_continue_poll_next() {
        let bytes1 = Ok(Bytes::from("hello".as_bytes()));
        let bytes2 = Ok(Bytes::from(" world".as_bytes()));
        let bytes3 = Ok(Bytes::from(", end test!".as_bytes()));

        let stream = futures_util::stream::iter(vec![bytes1, bytes2, bytes3]);

        let mut origin_stream = ClonedBytesStream::new(1024 * 64, Box::pin(stream));
        origin_stream.next().await;
        origin_stream.next().await;

        let mut clone_origin_stream = origin_stream.clone();

        drop(origin_stream);

        let cached_bytes = clone_origin_stream.next().await;
        assert!(cached_bytes.is_some());

        let cached_bytes = cached_bytes.unwrap();

        assert!(cached_bytes.is_ok());

        assert_eq!(bytes_data_to_str(cached_bytes.unwrap()), "hello world");

        let next_bytes = clone_origin_stream.next().await;

        assert!(next_bytes.is_some());

        let next_bytes = next_bytes.unwrap();

        assert!(next_bytes.is_ok());

        assert_eq!(bytes_data_to_str(next_bytes.unwrap()), ", end test!");

        let none_bytes = clone_origin_stream.next().await;
        assert!(none_bytes.is_none());
    }

    #[tokio::test]
    async fn test_cloned_bytes_stream_reached_max_capacity() {
        let bytes1 = Ok(Bytes::from("hello".as_bytes()));
        let bytes2 = Ok(Bytes::from(" world".as_bytes()));
        let bytes3 = Ok(Bytes::from(", end test!".as_bytes()));

        let stream = futures_util::stream::iter(vec![bytes1, bytes2, bytes3]);

        let mut origin_stream = ClonedBytesStream::new(5, Box::pin(stream));
        let hello_bytes = origin_stream.next().await;

        assert!(hello_bytes.is_some());

        let hello_bytes = hello_bytes.unwrap();

        assert!(hello_bytes.is_ok());

        assert_eq!(bytes_data_to_str(hello_bytes.unwrap()), "hello");

        let world_bytes = origin_stream.next().await;

        assert!(world_bytes.is_some());

        let world_bytes = world_bytes.unwrap();

        assert!(world_bytes.is_ok());

        assert_eq!(bytes_data_to_str(world_bytes.unwrap()), " world");

        let mut cloned_origin_stream = origin_stream.clone();

        drop(origin_stream);

        let end_bytes = cloned_origin_stream.next().await;

        assert!(end_bytes.is_some());

        let end_bytes = end_bytes.unwrap();

        assert!(end_bytes.is_ok());

        assert_eq!(bytes_data_to_str(end_bytes.unwrap()), ", end test!");

        let none_bytes = cloned_origin_stream.next().await;
        assert!(none_bytes.is_none());
    }

    fn bytes_data_to_str(mut bytes_data: BytesData) -> String {
        let len = bytes_data.remaining();
        let bytes = bytes_data.copy_to_bytes(len);
        String::from_utf8(bytes.to_vec()).unwrap()
    }
}

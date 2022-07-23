/*
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.
 * The ASF licenses this file to You under the Apache License, Version 2.0
 * (the "License"); you may not use this file except in compliance with
 * the License.  You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::{pin::Pin, task::Poll};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures_util::Stream;
use futures_util::{future, ready};
use http_body::Body;
use tonic::metadata::MetadataMap;

use crate::codec::{DecodeBuf, Decoder};

type BoxBody = http_body::combinators::UnsyncBoxBody<Bytes, tonic::Status>;

pub struct Streaming<T> {
    state: State,
    body: BoxBody,
    decoder: Box<dyn Decoder<Item = T, Error = tonic::Status> + Send + 'static>,
    buf: BytesMut,
    trailers: Option<MetadataMap>,
}

#[derive(PartialEq)]
enum State {
    ReadHeader,
    ReadBody { len: usize },
    Error,
}

impl<T> Streaming<T> {
    pub fn new<B, D>(body: B, decoder: D) -> Self
    where
        B: Body + Send + 'static,
        B::Error: Into<crate::Error>,
        D: Decoder<Item = T, Error = tonic::Status> + Send + 'static,
    {
        Self {
            state: State::ReadHeader,
            body: body
                .map_data(|mut buf| buf.copy_to_bytes(buf.remaining()))
                .map_err(|_err| tonic::Status::internal("err"))
                .boxed_unsync(),
            decoder: Box::new(decoder),
            buf: BytesMut::with_capacity(super::consts::BUFFER_SIZE),
            trailers: None,
        }
    }

    pub async fn message(&mut self) -> Result<Option<T>, tonic::Status> {
        match future::poll_fn(|cx| Pin::new(&mut *self).poll_next(cx)).await {
            Some(Ok(res)) => Ok(Some(res)),
            Some(Err(err)) => Err(err),
            None => Ok(None),
        }
    }

    pub async fn trailer(&mut self) -> Result<Option<MetadataMap>, tonic::Status> {
        if let Some(t) = self.trailers.take() {
            return Ok(Some(t));
        }
        // while self.message().await?.is_some() {}

        let trailer = future::poll_fn(|cx| Pin::new(&mut self.body).poll_trailers(cx)).await;
        trailer.map(|data| data.map(MetadataMap::from_headers))
    }

    pub fn decode_chunk(&mut self) -> Result<Option<T>, tonic::Status> {
        if self.state == State::ReadHeader {
            // buffer is full
            if self.buf.remaining() < super::consts::HEADER_SIZE {
                return Ok(None);
            }

            let _is_compressed = self.buf.get_u8();
            let len = self.buf.get_u32() as usize;
            self.buf.reserve(len as usize);

            self.state = State::ReadBody { len }
        }

        if let State::ReadBody { len } = self.state {
            if self.buf.remaining() < len || self.buf.len() < len {
                return Ok(None);
            }

            let decoding_result = self.decoder.decode(&mut DecodeBuf::new(&mut self.buf, len));

            return match decoding_result {
                Ok(Some(r)) => {
                    self.state = State::ReadHeader;
                    Ok(Some(r))
                }
                Ok(None) => Ok(None),
                Err(err) => Err(err),
            };
        }

        Ok(None)
    }
}

impl<T> Stream for Streaming<T> {
    type Item = Result<T, tonic::Status>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        loop {
            if self.state == State::Error {
                return Poll::Ready(None);
            }

            if let Some(item) = self.decode_chunk()? {
                return Poll::Ready(Some(Ok(item)));
            }

            let chunk = match ready!(Pin::new(&mut self.body).poll_data(cx)) {
                Some(Ok(d)) => Some(d),
                Some(Err(e)) => {
                    let _ = std::mem::replace(&mut self.state, State::Error);
                    let err: crate::Error = e.into();
                    return Poll::Ready(Some(Err(tonic::Status::internal(err.to_string()))));
                }
                None => None,
            };

            if let Some(data) = chunk {
                self.buf.put(data)
            } else {
                break;
            }
        }

        match ready!(Pin::new(&mut self.body).poll_trailers(cx)) {
            Ok(trailer) => {
                self.trailers = trailer.map(MetadataMap::from_headers);
            }
            Err(err) => {
                println!("poll_trailers, err: {}", err);
            }
        }

        Poll::Ready(None)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

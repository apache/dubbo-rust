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

use crate::status::Status;
use bytes::{BufMut, Bytes, BytesMut};
use futures_core::{Stream, TryStream};
use futures_util::{ready, StreamExt, TryStreamExt};
use http_body::Body;
use pin_project::pin_project;

use super::compression::{compress, CompressionEncoding};
use crate::triple::codec::{EncodeBuf, Encoder};

#[allow(unused_must_use)]
pub fn encode<E, B>(
    mut encoder: Box<dyn Encoder<Error = Status, Item = E> + Send + 'static>,
    resp_body: B,
    compression_encoding: Option<CompressionEncoding>,
    encode_as_grpc: bool,
) -> impl TryStream<Ok = Bytes, Error = Status>
where
    B: Stream<Item = Result<E, Status>>,
{
    async_stream::stream! {
        let mut buf = BytesMut::with_capacity(super::consts::BUFFER_SIZE);
        futures_util::pin_mut!(resp_body);

        let (enable_compress, mut uncompression_buf) = match compression_encoding {
            Some(CompressionEncoding::Gzip) => (true, BytesMut::with_capacity(super::consts::BUFFER_SIZE)),
            None => (false, BytesMut::new())
        };

        loop {
            match resp_body.next().await {
                Some(Ok(item)) => {
                    if encode_as_grpc {
                        buf.reserve(super::consts::HEADER_SIZE);
                        unsafe {
                            buf.advance_mut(super::consts::HEADER_SIZE);
                        }
                    }
                    // 编码数据到缓冲中
                    if enable_compress {
                        uncompression_buf.clear();

                        encoder.encode(item, &mut EncodeBuf::new(&mut uncompression_buf))
                            .map_err(|_e| crate::status::Status::new(crate::status::Code::Internal, "encode error".to_string()));

                        let len = uncompression_buf.len();
                        compress(compression_encoding.unwrap(), &mut uncompression_buf, &mut buf, len)
                            .map_err(|_| crate::status::Status::new(crate::status::Code::Internal, "compress error".to_string()));
                    } else {
                        encoder.encode(item, &mut EncodeBuf::new(&mut buf)).map_err(|_e| crate::status::Status::new(crate::status::Code::Internal, "encode error".to_string()));
                    }
                    let result=match encode_as_grpc{
                        true=>{
                            let len = buf.len() - super::consts::HEADER_SIZE;
                            {
                                let mut buf = &mut buf[..super::consts::HEADER_SIZE];
                                buf.put_u8(enable_compress as u8);
                                buf.put_u32(len as u32);
                            }
                            buf.split_to(len + super::consts::HEADER_SIZE)
                        }
                        false=>{
                            buf.clone()
                        }
                    };
                    yield Ok(result.freeze());
                },
                Some(Err(err)) => yield Err(err.into()),
                None => break,
            }
        }
    }
}

pub fn encode_server<E, B>(
    encoder: Box<dyn Encoder<Error = Status, Item = E> + Send + 'static>,
    body: B,
    compression_encoding: Option<CompressionEncoding>,
    encode_as_grpc: bool,
) -> EncodeBody<impl Stream<Item = Result<Bytes, Status>>>
where
    B: Stream<Item = Result<E, Status>>,
{
    let s = encode(encoder, body, compression_encoding, encode_as_grpc).into_stream();
    EncodeBody::new_server(s)
}

pub fn encode_client<E, B>(
    encoder: Box<dyn Encoder<Error = Status, Item = E> + Send + 'static>,
    body: B,
    compression_encoding: Option<CompressionEncoding>,
    is_grpc: bool,
) -> EncodeBody<impl Stream<Item = Result<Bytes, Status>>>
where
    B: Stream<Item = E>,
{
    let s = encode(encoder, body.map(Ok), compression_encoding, is_grpc).into_stream();
    EncodeBody::new_client(s)
}

#[derive(Debug)]
enum Role {
    Server,
    Client,
}

#[pin_project]
pub struct EncodeBody<S> {
    #[pin]
    inner: S,
    role: Role,
    is_end_stream: bool,
    error: Option<crate::status::Status>,
}

impl<S> EncodeBody<S> {
    pub fn new_server(inner: S) -> Self {
        Self {
            inner,
            role: Role::Server,
            is_end_stream: false,
            error: None,
        }
    }

    pub fn new_client(inner: S) -> Self {
        Self {
            inner,
            role: Role::Client,
            is_end_stream: false,
            error: None,
        }
    }
}

impl<S> Body for EncodeBody<S>
where
    S: Stream<Item = Result<Bytes, crate::status::Status>>,
{
    type Data = Bytes;

    type Error = crate::status::Status;

    fn is_end_stream(&self) -> bool {
        self.is_end_stream
    }

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let mut self_proj = self.project();
        match ready!(self_proj.inner.try_poll_next_unpin(cx)) {
            Some(Ok(d)) => Some(Ok(d)).into(),
            Some(Err(status)) => {
                *self_proj.error = Some(status);
                None.into()
            }
            None => None.into(),
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<Option<http::HeaderMap>, Self::Error>> {
        let self_proj = self.project();
        if *self_proj.is_end_stream {
            return Poll::Ready(Ok(None));
        }

        let status = if let Some(status) = self_proj.error.take() {
            *self_proj.is_end_stream = true;
            status
        } else {
            crate::status::Status::new(
                crate::status::Code::Ok,
                "poll trailer successfully.".to_string(),
            )
        };
        let http = status.to_http();

        Poll::Ready(Ok(Some(http.headers().to_owned())))
    }
}

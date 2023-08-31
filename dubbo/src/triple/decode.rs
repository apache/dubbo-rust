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
use dubbo_logger::tracing;
use futures_util::{future, ready, Stream};
use http_body::Body;

use super::compression::{decompress, CompressionEncoding};
use crate::{
    invocation::Metadata,
    triple::codec::{DecodeBuf, Decoder},
};

type BoxBody = http_body::combinators::UnsyncBoxBody<Bytes, crate::status::Status>;

pub struct Decoding<T> {
    state: State,
    body: BoxBody,
    decoder: Box<dyn Decoder<Item = T, Error = crate::status::Status> + Send + 'static>,
    buf: BytesMut,
    trailers: Option<Metadata>,
    compress: Option<CompressionEncoding>,
    decompress_buf: BytesMut,
    decode_as_grpc: bool,
}

#[derive(PartialEq)]
enum State {
    ReadHeader,
    ReadHttpBody,
    ReadBody { len: usize, is_compressed: bool },
    Error,
}

impl<T> Decoding<T> {
    pub fn new<B>(
        body: B,
        decoder: Box<dyn Decoder<Item = T, Error = crate::status::Status> + Send + 'static>,
        compress: Option<CompressionEncoding>,
        decode_as_grpc: bool,
    ) -> Self
    where
        B: Body + Send + 'static,
        B::Error: Into<crate::Error>,
    {
        //Determine whether to use the gRPC mode to handle request data
        Self {
            state: State::ReadHeader,
            body: body
                .map_data(|mut buf| buf.copy_to_bytes(buf.remaining()))
                .map_err(|_err| {
                    crate::status::Status::new(
                        crate::status::Code::Internal,
                        "internal decode err".to_string(),
                    )
                })
                .boxed_unsync(),
            decoder,
            buf: BytesMut::with_capacity(super::consts::BUFFER_SIZE),
            trailers: None,
            compress,
            decompress_buf: BytesMut::new(),
            decode_as_grpc,
        }
    }

    pub async fn message(&mut self) -> Result<Option<T>, crate::status::Status> {
        match future::poll_fn(|cx| Pin::new(&mut *self).poll_next(cx)).await {
            Some(Ok(res)) => Ok(Some(res)),
            Some(Err(err)) => Err(err),
            None => Ok(None),
        }
    }

    pub async fn trailer(&mut self) -> Result<Option<Metadata>, crate::status::Status> {
        if let Some(t) = self.trailers.take() {
            return Ok(Some(t));
        }
        // while self.message().await?.is_some() {}

        let trailer = future::poll_fn(|cx| Pin::new(&mut self.body).poll_trailers(cx)).await;
        trailer.map(|data| data.map(Metadata::from_headers))
    }

    pub fn decode_http(&mut self) -> Result<Option<T>, crate::status::Status> {
        if self.state == State::ReadHeader {
            self.state = State::ReadHttpBody;
            return Ok(None);
        }
        if let State::ReadHttpBody = self.state {
            if self.buf.is_empty() {
                return Ok(None);
            }
            match self.compress {
                None => self.decompress_buf = self.buf.clone(),
                Some(compress) => {
                    let len = self.buf.len();
                    if let Err(err) =
                        decompress(compress, &mut self.buf, &mut self.decompress_buf, len)
                    {
                        return Err(crate::status::Status::new(
                            crate::status::Code::Internal,
                            err.to_string(),
                        ));
                    }
                }
            }
            let len = self.decompress_buf.len();
            let decoding_result = self
                .decoder
                .decode(&mut DecodeBuf::new(&mut self.decompress_buf, len));

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

    pub fn decode_grpc(&mut self) -> Result<Option<T>, crate::status::Status> {
        if self.state == State::ReadHeader {
            // buffer is full
            if self.buf.remaining() < super::consts::HEADER_SIZE {
                return Ok(None);
            }

            let is_compressed = match self.buf.get_u8() {
                0 => false,
                1 => {
                    if self.compress.is_some() {
                        true
                    } else {
                        return Err(crate::status::Status::new(
                            crate::status::Code::Internal,
                            "set compression flag, but no grpc-encoding specified".to_string(),
                        ));
                    }
                }
                v => {
                    return Err(crate::status::Status::new(
                        crate::status::Code::Internal,
                        format!(
                            "receive message with compression flag{}, flag should be 0 or 1",
                            v
                        ),
                    ))
                }
            };
            let len = self.buf.get_u32() as usize;
            self.buf.reserve(len as usize);

            self.state = State::ReadBody { len, is_compressed }
        }

        if let State::ReadBody { len, is_compressed } = self.state {
            if self.buf.remaining() < len || self.buf.len() < len {
                return Ok(None);
            }

            let decoding_result = if is_compressed {
                self.decompress_buf.clear();
                if let Err(err) = decompress(
                    self.compress.unwrap(),
                    &mut self.buf,
                    &mut self.decompress_buf,
                    len,
                ) {
                    return Err(crate::status::Status::new(
                        crate::status::Code::Internal,
                        err.to_string(),
                    ));
                }

                let decompress_len = self.decompress_buf.len();
                self.decoder.decode(&mut DecodeBuf::new(
                    &mut self.decompress_buf,
                    decompress_len,
                ))
            } else {
                self.decoder.decode(&mut DecodeBuf::new(&mut self.buf, len))
            };

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

    pub fn decode_chunk(&mut self) -> Result<Option<T>, crate::status::Status> {
        if self.decode_as_grpc {
            self.decode_grpc()
        } else {
            self.decode_http()
        }
    }
}

impl<T> Stream for Decoding<T> {
    type Item = Result<T, crate::status::Status>;

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
                    return Poll::Ready(Some(Err(crate::status::Status::new(
                        crate::status::Code::Internal,
                        err.to_string(),
                    ))));
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
                self.trailers = trailer.map(Metadata::from_headers);
            }
            Err(err) => {
                tracing::error!("poll_trailers, err: {}", err);
            }
        }

        Poll::Ready(None)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

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
use futures_util::{ ready, Stream};
use http_body::Body;
use crate::{
    triple::codec::{DecodeBuf, Decoder},
};

type BoxBody = http_body::combinators::UnsyncBoxBody<Bytes, crate::status::Status>;

pub struct DecodingJSON<T> {
    state: State,
    body: BoxBody,
    decoder: Box<dyn Decoder<Item=T, Error=crate::status::Status> + Send + 'static>,
    buf: BytesMut,
    // decompress_buf: BytesMut,
}

#[derive(PartialEq)]
enum State {
    Prepare,
    ReadBody,
    Ok,
    Error,
}

impl<T> DecodingJSON<T> {
    pub fn new<B, D>(body: B, decoder: D) -> Self
        where
            B: Body + Send + 'static,
            B::Error: Into<crate::Error>,
            D: Decoder<Item=T, Error=crate::status::Status> + Send + 'static,
    {
        Self {
            state: State::Prepare,
            body: body
                .map_data(|mut buf| buf.copy_to_bytes(buf.remaining()))
                .map_err(|_err| {
                    crate::status::Status::new(
                        crate::status::Code::Internal,
                        "internal decode err".to_string(),
                    )
                })
                .boxed_unsync(),
            decoder: Box::new(decoder),
            buf: BytesMut::with_capacity(super::consts::BUFFER_SIZE),
            // decompress_buf: BytesMut::new(),
        }
    }


    pub fn decode_chunk(&mut self) -> Result<Option<T>, crate::status::Status> {
        if self.state == State::Prepare {
            self.state = State::ReadBody;
            return Ok(None)
        }

        if let State::ReadBody = self.state {
            let decoding_result =self.decoder.decode(&mut DecodeBuf::new(&mut self.buf, super::consts::BUFFER_SIZE));

            return match decoding_result {
                Ok(Some(r)) => {
                    self.state = State::Ok;
                    Ok(Some(r))
                }
                Ok(None) => Ok(None),
                Err(err) => Err(err),
            };
        }

        Ok(None)
    }
}

impl<T> Stream for DecodingJSON<T> {
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

        Poll::Ready(None)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

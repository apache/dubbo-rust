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
use crate::status::Status;
use bytes::{Bytes, BytesMut};
use futures_core::{Stream, TryStream};
use futures_util::{StreamExt, TryStreamExt};

use crate::triple::codec::{EncodeBuf, Encoder};
use crate::triple::compression::{compress, CompressionEncoding};
use crate::triple::encode::EncodeBody;

#[allow(unused_must_use)]
pub fn encode_json<E, B>(
    mut encoder: E,
    resp_body: B,
    compression_encoding: Option<CompressionEncoding>,
) -> impl TryStream<Ok=Bytes, Error=Status>
    where
        E: Encoder<Error=Status>,
        B: Stream<Item=Result<E::Item, Status>>,
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
                    if enable_compress {
                        uncompression_buf.clear();
                        encoder.encode(item, &mut EncodeBuf::new(&mut uncompression_buf))
                            .map_err(|_e| crate::status::Status::new(crate::status::Code::Internal, "encode error".to_string()));

                        let len = uncompression_buf.len();
                        compress(compression_encoding.unwrap(), &mut uncompression_buf, &mut buf, len)
                            .map_err(|_| crate::status::Status::new(crate::status::Code::Internal, "compress error".to_string()));
                    } else {
                        encoder.encode(item, &mut EncodeBuf::new(&mut buf)).map_err(|_e| crate::status::Status::new(crate::status::Code::Internal, "encode error".to_string()));
                    };
                    yield Ok(buf.clone().freeze());
                },
                Some(Err(err)) => yield Err(err.into()),
                None => break,
            }
        }
    }
}

pub fn encode_server_json<E, B>(
    encoder: E,
    body: B,
    compress: Option<CompressionEncoding>,
) -> EncodeBody<impl Stream<Item=Result<Bytes, Status>>>
    where
        E: Encoder<Error=Status>,
        B: Stream<Item=Result<E::Item, Status>>,
{
    let s = encode_json(encoder, body, compress).into_stream();
    EncodeBody::new_server(s)
}
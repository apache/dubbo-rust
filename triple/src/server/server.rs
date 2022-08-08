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

use futures_util::{future, stream, StreamExt, TryStreamExt};
use http_body::Body;

use super::compression::{CompressionEncoding, COMPRESSIONS};
use crate::codec::Codec;
use crate::invocation::Request;
use crate::server::encode::encode_server;
use crate::server::service::{ClientStreamingSvc, StreamingSvc, UnaryService};
use crate::server::Decoding;
use crate::BoxBody;
use config::BusinessConfig;

pub const GRPC_ACCEPT_ENCODING: &str = "grpc-accept-encoding";
pub const GRPC_ENCODING: &str = "grpc-encoding";

pub struct TripleServer<T> {
    codec: T,
    compression: Option<CompressionEncoding>,
}

impl<T> TripleServer<T> {
    pub fn new(codec: T) -> Self {
        Self {
            codec,
            compression: None,
        }
    }
}

impl<T> TripleServer<T>
where
    T: Codec,
{
    pub async fn client_streaming<S, B>(
        &mut self,
        mut service: S,
        req: http::Request<B>,
    ) -> http::Response<BoxBody>
    where
        S: ClientStreamingSvc<T::Decode, Response = T::Encode>,
        B: Body + Send + 'static,
        B::Error: Into<crate::Error> + Send,
    {
        let mut accept_encoding = CompressionEncoding::from_accept_encoding(req.headers());
        if self.compression.is_none() || accept_encoding.is_none() {
            accept_encoding = None;
        }

        // Get grpc_encoding from http_header, decompress message.
        let compression = match self.get_encoding_from_req(req.headers()) {
            Ok(val) => val,
            Err(status) => return status.to_http(),
        };

        let req_stream = req.map(|body| Decoding::new(body, self.codec.decoder(), compression));

        let resp = service.call(Request::from_http(req_stream)).await;

        let (mut parts, resp_body) = resp.unwrap().into_http().into_parts();
        let resp_body = encode_server(
            self.codec.encoder(),
            stream::once(future::ready(resp_body)).map(Ok).into_stream(),
            accept_encoding,
        );

        parts.headers.insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("application/grpc"),
        );
        if let Some(encoding) = accept_encoding {
            parts
                .headers
                .insert(GRPC_ENCODING, encoding.into_header_value());
        }
        parts.status = http::StatusCode::OK;
        http::Response::from_parts(parts, BoxBody::new(resp_body))
    }
    pub async fn bidi_streaming<S, B>(
        &mut self,
        mut service: S,
        req: http::Request<B>,
    ) -> http::Response<BoxBody>
    where
        S: StreamingSvc<T::Decode, Response = T::Encode>,
        S::ResponseStream: Send + 'static,
        B: Body + Send + 'static,
        B::Error: Into<crate::Error> + Send,
    {
        // Firstly, get grpc_accept_encoding from http_header, get compression
        // Secondly, if server enable compression and compression is valid, this method should compress response
        let mut accept_encoding = CompressionEncoding::from_accept_encoding(req.headers());
        if self.compression.is_none() || accept_encoding.is_none() {
            accept_encoding = None;
        }

        // Get grpc_encoding from http_header, decompress message.
        let compression = match self.get_encoding_from_req(req.headers()) {
            Ok(val) => val,
            Err(status) => return status.to_http(),
        };

        let req_stream = req.map(|body| Decoding::new(body, self.codec.decoder(), compression));

        let resp = service.call(Request::from_http(req_stream)).await;

        let (mut parts, resp_body) = resp.unwrap().into_http().into_parts();
        let resp_body = encode_server(self.codec.encoder(), resp_body, compression);

        parts.headers.insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("application/grpc"),
        );
        if let Some(encoding) = accept_encoding {
            parts
                .headers
                .insert(GRPC_ENCODING, encoding.into_header_value());
        }
        parts.status = http::StatusCode::OK;
        http::Response::from_parts(parts, BoxBody::new(resp_body))
    }

    pub async fn unary<S, B>(
        &mut self,
        mut service: S,
        req: http::Request<B>,
    ) -> http::Response<BoxBody>
    where
        S: UnaryService<T::Decode, Response = T::Encode>,
        B: Body + Send + 'static,
        B::Error: Into<crate::Error> + Send,
    {
        let mut accept_encoding = CompressionEncoding::from_accept_encoding(req.headers());
        if self.compression.is_none() || accept_encoding.is_none() {
            accept_encoding = None;
        }

        let compression = match self.get_encoding_from_req(req.headers()) {
            Ok(val) => val,
            Err(status) => return status.to_http(),
        };

        let req_stream = req.map(|body| Decoding::new(body, self.codec.decoder(), compression));
        let (parts, mut body) = Request::from_http(req_stream).into_parts();
        let msg = body
            .try_next()
            .await
            .unwrap()
            .ok_or_else(|| tonic::Status::new(tonic::Code::Unknown, "request wrong"));

        let resp = service.call(Request::from_parts(parts, msg.unwrap())).await;

        let (mut parts, resp_body) = resp.unwrap().into_http().into_parts();
        let resp_body = encode_server(
            self.codec.encoder(),
            stream::once(future::ready(resp_body)).map(Ok).into_stream(),
            accept_encoding,
        );

        parts.headers.insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("application/grpc"),
        );
        if let Some(encoding) = accept_encoding {
            parts
                .headers
                .insert(GRPC_ENCODING, encoding.into_header_value());
        }
        parts.status = http::StatusCode::OK;
        http::Response::from_parts(parts, BoxBody::new(resp_body))
    }

    fn get_encoding_from_req(
        &self,
        header: &http::HeaderMap,
    ) -> Result<Option<CompressionEncoding>, tonic::Status> {
        let encoding = header.get(GRPC_ENCODING).unwrap().to_str().unwrap();
        let compression = match COMPRESSIONS.get(encoding) {
            Some(val) => val.to_owned(),
            None => {
                let mut status = tonic::Status::unimplemented(format!(
                    "grpc-accept-encoding: {} not support!",
                    encoding
                ));

                status.metadata_mut().insert(
                    GRPC_ACCEPT_ENCODING,
                    tonic::metadata::MetadataValue::from_static("gzip,identity"),
                );

                return Err(status);
            }
        };
        Ok(compression)
    }
}

impl<T> BusinessConfig for TripleServer<T> {
    fn init() -> Self {
        todo!()
    }

    fn load() -> Result<(), std::convert::Infallible> {
        todo!()
    }
}

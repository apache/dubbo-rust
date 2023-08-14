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

use std::marker::PhantomData;
use futures_util::{future, stream, StreamExt, TryStreamExt};
use http::HeaderValue;
use http_body::Body;
use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{
    invocation::Request,
    triple::{
        codec::Codec,
        compression::{CompressionEncoding, COMPRESSIONS},
        decode::Decoding,
        encode::encode_server,
        server::service::{ClientStreamingSvc, ServerStreamingSvc, StreamingSvc, UnarySvc},
    },
    BoxBody,
};
use dubbo_config::BusinessConfig;
use crate::codegen::{ProstCodec, SerdeCodec};
use crate::triple::encode_json::encode_server_json;

pub const GRPC_ACCEPT_ENCODING: &str = "grpc-accept-encoding";
pub const GRPC_ENCODING: &str = "grpc-encoding";

pub struct TripleServer<T, V>
{
    _pd: PhantomData<(T, V)>,
    compression: Option<CompressionEncoding>,
}

impl<T, V> TripleServer<T, V> {
    pub fn new() -> Self {
        Self {
            _pd: PhantomData,
            compression: Some(CompressionEncoding::Gzip),
        }
    }
}

impl<T, V> TripleServer<T, V>
    where
        T: Message + for<'a> Deserialize<'a> + Default + 'static,
        V: Message + Serialize + Default + 'static
{
    pub async fn client_streaming<S, B>(
        &mut self,
        mut service: S,
        req: http::Request<B>,
    ) -> http::Response<BoxBody>
        where
            S: ClientStreamingSvc<T, Response=V>,
            B: Body + Send + 'static,
            B::Error: Into<crate::Error> + Send,
    {
        let is_json = req.headers().get("content-type").unwrap() == "application/grpc+json";
        if is_json {
            let mut codec = SerdeCodec::<V, T>::default();
            let mut accept_encoding = CompressionEncoding::from_accept_encoding(req.headers());
            if self.compression.is_none() || accept_encoding.is_none() {
                accept_encoding = None;
            }

            // Get grpc_encoding from http_header, decompress message.
            let compression = match self.get_encoding_from_req(req.headers()) {
                Ok(val) => val,
                Err(status) => return status.to_http(),
            };

            let req_stream = req.map(|body| Decoding::new(body, codec.decoder(), compression, true));

            let resp = service.call(Request::from_http(req_stream)).await;

            let (mut parts, resp_body) = match resp {
                Ok(v) => v.into_http().into_parts(),
                Err(err) => return err.to_http(),
            };

            let resp_body = encode_server_json(
                codec.encoder(),
                stream::once(future::ready(resp_body)).map(Ok).into_stream(),
                accept_encoding,
            );

            parts.headers.insert(
                http::header::CONTENT_TYPE,
                http::HeaderValue::from_static("application/grpc+json"),
            );
            if let Some(encoding) = accept_encoding {
                parts
                    .headers
                    .insert(GRPC_ENCODING, encoding.into_header_value());
            }
            parts.status = http::StatusCode::OK;
            http::Response::from_parts(parts, BoxBody::new(resp_body))
        } else {
            let mut codec = ProstCodec::<V, T>::default();
            let mut accept_encoding = CompressionEncoding::from_accept_encoding(req.headers());
            if self.compression.is_none() || accept_encoding.is_none() {
                accept_encoding = None;
            }

            // Get grpc_encoding from http_header, decompress message.
            let compression = match self.get_encoding_from_req(req.headers()) {
                Ok(val) => val,
                Err(status) => return status.to_http(),
            };

            let req_stream = req.map(|body| Decoding::new(body, codec.decoder(), compression, false));

            let resp = service.call(Request::from_http(req_stream)).await;

            let (mut parts, resp_body) = match resp {
                Ok(v) => v.into_http().into_parts(),
                Err(err) => return err.to_http(),
            };

            let resp_body = encode_server(
                codec.encoder(),
                stream::once(future::ready(resp_body)).map(Ok).into_stream(),
                accept_encoding,
            );

            parts.headers.insert(
                http::header::CONTENT_TYPE,
                http::HeaderValue::from_static("application/grpc+proto"),
            );
            if let Some(encoding) = accept_encoding {
                parts
                    .headers
                    .insert(GRPC_ENCODING, encoding.into_header_value());
            }
            parts.status = http::StatusCode::OK;
            http::Response::from_parts(parts, BoxBody::new(resp_body))
        }
    }

    pub async fn bidi_streaming<S, B>(
        &mut self,
        mut service: S,
        req: http::Request<B>,
    ) -> http::Response<BoxBody>
        where
            S: StreamingSvc<T, Response=V>,
            S::ResponseStream: Send + 'static,
            B: Body + Send + 'static,
            B::Error: Into<crate::Error> + Send,
    {
        let is_json = req.headers().get("content-type").unwrap() == "application/grpc+json";
        if is_json {
            let mut codec = SerdeCodec::<V, T>::default();
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

            let req_stream = req.map(|body| Decoding::new(body, codec.decoder(), compression, is_json));

            let resp = service.call(Request::from_http(req_stream)).await;

            let (mut parts, resp_body) = match resp {
                Ok(v) => v.into_http().into_parts(),
                Err(err) => return err.to_http(),
            };
            let resp_body = encode_server_json(codec.encoder(), resp_body, compression);

            parts.headers.insert(
                http::header::CONTENT_TYPE,
                http::HeaderValue::from_static("application/grpc+json"),
            );
            if let Some(encoding) = accept_encoding {
                parts
                    .headers
                    .insert(GRPC_ENCODING, encoding.into_header_value());
            }
            parts.status = http::StatusCode::OK;
            http::Response::from_parts(parts, BoxBody::new(resp_body))
        } else {
            let mut codec = ProstCodec::<V, T>::default();
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

            let req_stream = req.map(|body| Decoding::new(body, codec.decoder(), compression, false));

            let resp = service.call(Request::from_http(req_stream)).await;

            let (mut parts, resp_body) = match resp {
                Ok(v) => v.into_http().into_parts(),
                Err(err) => return err.to_http(),
            };
            let resp_body = encode_server(codec.encoder(), resp_body, compression);

            parts.headers.insert(
                http::header::CONTENT_TYPE,
                http::HeaderValue::from_static("application/grpc+proto"),
            );
            if let Some(encoding) = accept_encoding {
                parts
                    .headers
                    .insert(GRPC_ENCODING, encoding.into_header_value());
            }
            parts.status = http::StatusCode::OK;
            http::Response::from_parts(parts, BoxBody::new(resp_body))
        }
    }

    pub async fn server_streaming<S, B>(
        &mut self,
        mut service: S,
        req: http::Request<B>,
    ) -> http::Response<BoxBody>
        where
            S: ServerStreamingSvc<T, Response=V>,
            S::ResponseStream: Send + 'static,
            B: Body + Send + 'static,
            B::Error: Into<crate::Error> + Send,
    {
        let is_json = req.headers().get("content-type").unwrap() == "application/grpc+json";
        if is_json {
            let mut codec = SerdeCodec::<V, T>::default();
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
            let req_stream = req.map(|body| Decoding::new(body, codec.decoder(), compression, true));
            let (parts, mut body) = Request::from_http(req_stream).into_parts();
            let msg = body.try_next().await.unwrap().ok_or_else(|| {
                crate::status::Status::new(crate::status::Code::Unknown, "request wrong".to_string())
            });
            let msg = match msg {
                Ok(v) => v,
                Err(err) => return err.to_http(),
            };

            let resp = service.call(Request::from_parts(parts, msg)).await;

            let (mut parts, resp_body) = match resp {
                Ok(v) => v.into_http().into_parts(),
                Err(err) => return err.to_http(),
            };
            let resp_body = encode_server_json(codec.encoder(), resp_body, compression);

            parts.headers.insert(
                http::header::CONTENT_TYPE,
                http::HeaderValue::from_static("application/grpc+json"),
            );
            if let Some(encoding) = accept_encoding {
                parts
                    .headers
                    .insert(GRPC_ENCODING, encoding.into_header_value());
            }
            parts.status = http::StatusCode::OK;
            http::Response::from_parts(parts, BoxBody::new(resp_body))
        } else {
            let mut codec = ProstCodec::<V, T>::default();
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
            let req_stream = req.map(|body| Decoding::new(body, codec.decoder(), compression, false));
            let (parts, mut body) = Request::from_http(req_stream).into_parts();
            let msg = body.try_next().await.unwrap().ok_or_else(|| {
                crate::status::Status::new(crate::status::Code::Unknown, "request wrong".to_string())
            });
            let msg = match msg {
                Ok(v) => v,
                Err(err) => return err.to_http(),
            };

            let resp = service.call(Request::from_parts(parts, msg)).await;

            let (mut parts, resp_body) = match resp {
                Ok(v) => v.into_http().into_parts(),
                Err(err) => return err.to_http(),
            };
            let resp_body = encode_server(codec.encoder(), resp_body, compression);

            parts.headers.insert(
                http::header::CONTENT_TYPE,
                http::HeaderValue::from_static("application/grpc+proto"),
            );
            if let Some(encoding) = accept_encoding {
                parts
                    .headers
                    .insert(GRPC_ENCODING, encoding.into_header_value());
            }
            parts.status = http::StatusCode::OK;
            http::Response::from_parts(parts, BoxBody::new(resp_body))
        }
    }

    pub async fn unary<S, B>(
        &mut self,
        mut service: S,
        req: http::Request<B>,
    ) -> http::Response<BoxBody>
        where
            S: UnarySvc<T, Response=V>,
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
        let content_type = req.headers().get("content-type").cloned().unwrap_or(HeaderValue::from_str("application/json").unwrap());
        let is_json = content_type == "application/json" || content_type == "application/grpc+json";
        if is_json {
            let mut codec = SerdeCodec::<V, T>::default();
            let req_stream = req.map(|body| Decoding::new(body, codec.decoder(), compression, is_json));
            let (parts, mut body) = Request::from_http(req_stream).into_parts();
            let msg = body.try_next().await.unwrap().ok_or_else(|| {
                crate::status::Status::new(crate::status::Code::Unknown, "request wrong".to_string())
            });
            let msg = match msg {
                Ok(v) => v,
                Err(err) => return err.to_http(),
            };

            let resp = service.call(Request::from_parts(parts, msg)).await;

            let (mut parts, resp_body) = match resp {
                Ok(v) => v.into_http().into_parts(),
                Err(err) => return err.to_http(),
            };
            let resp_body = encode_server_json(
                codec.encoder(),
                stream::once(future::ready(resp_body)).map(Ok).into_stream(),
                accept_encoding,
            );

            parts.headers.insert(
                http::header::CONTENT_TYPE,
                content_type,
            );
            if let Some(encoding) = accept_encoding {
                parts
                    .headers
                    .insert(GRPC_ENCODING, encoding.into_header_value());
            }
            parts.status = http::StatusCode::OK;
            http::Response::from_parts(parts, BoxBody::new(resp_body))
        }else {
            let mut codec = ProstCodec::<V, T>::default();
            let req_stream = req.map(|body| Decoding::new(body, codec.decoder(), compression, is_json));
            let (parts, mut body) = Request::from_http(req_stream).into_parts();
            let msg = body.try_next().await.unwrap().ok_or_else(|| {
                crate::status::Status::new(crate::status::Code::Unknown, "request wrong".to_string())
            });
            let msg = match msg {
                Ok(v) => v,
                Err(err) => return err.to_http(),
            };

            let resp = service.call(Request::from_parts(parts, msg)).await;

            let (mut parts, resp_body) = match resp {
                Ok(v) => v.into_http().into_parts(),
                Err(err) => return err.to_http(),
            };
            let resp_body = encode_server(
                codec.encoder(),
                stream::once(future::ready(resp_body)).map(Ok).into_stream(),
                accept_encoding,
            );

            parts.headers.insert(
                http::header::CONTENT_TYPE,
                http::HeaderValue::from_static("application/grpc+proto"),
            );
            if let Some(encoding) = accept_encoding {
                parts
                    .headers
                    .insert(GRPC_ENCODING, encoding.into_header_value());
            }
            parts.status = http::StatusCode::OK;
            http::Response::from_parts(parts, BoxBody::new(resp_body))
        }
    }

    fn get_encoding_from_req(
        &self,
        header: &http::HeaderMap,
    ) -> Result<Option<CompressionEncoding>, crate::status::Status> {
        let encoding = match header.get(GRPC_ENCODING) {
            Some(val) => val.to_str().unwrap(),
            None => return Ok(None),
        };

        let compression = match COMPRESSIONS.get(encoding) {
            Some(val) => val.to_owned(),
            None => {
                let status = crate::status::Status::new(
                    crate::status::Code::Unimplemented,
                    format!("grpc-accept-encoding: {} not support!", encoding),
                );

                return Err(status);
            }
        };
        Ok(compression)
    }
}

impl<T, V> BusinessConfig for TripleServer<T, V> {
    fn init() -> Self {
        todo!()
    }

    fn load() -> Result<(), std::convert::Infallible> {
        todo!()
    }
}

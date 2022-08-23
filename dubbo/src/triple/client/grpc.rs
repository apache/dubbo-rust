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

use std::str::FromStr;

use futures_util::{future, stream, StreamExt, TryStreamExt};
use http::HeaderValue;

use crate::triple::codec::Codec;
use crate::triple::invocation::{IntoStreamingRequest, Request, Response, Metadata};
use crate::triple::server::compression::CompressionEncoding;
use crate::triple::server::encode::encode;
use crate::triple::server::Decoding;

#[derive(Debug, Clone, Default)]
pub struct TripleClient {
    host: Option<http::Uri>,
    inner: ConnectionPool,
    send_compression_encoding: Option<CompressionEncoding>,
}

#[derive(Debug, Default, Clone)]
pub struct ConnectionPool {
    http2_only: bool,
}

impl ConnectionPool {
    pub fn new() -> Self {
        ConnectionPool { http2_only: true }
    }

    pub fn builder(self) -> hyper::Client<hyper::client::HttpConnector> {
        hyper::Client::builder()
            .http2_only(self.http2_only)
            .build_http()
    }
}

// TODO: initial connection pool
impl TripleClient {
    pub fn new() -> Self {
        TripleClient {
            host: None,
            inner: ConnectionPool::new(),
            send_compression_encoding: Some(CompressionEncoding::Gzip),
        }
    }

    /// host: http://0.0.0.0:8888
    pub fn with_host(self, host: String) -> Self {
        let uri = http::Uri::from_str(&host).unwrap();
        TripleClient {
            host: Some(uri),
            ..self
        }
    }
}

impl TripleClient {
    fn map_request(
        &self,
        path: http::uri::PathAndQuery,
        body: hyper::Body,
    ) -> http::Request<hyper::Body> {
        let mut parts = self.host.clone().unwrap().into_parts();
        parts.path_and_query = Some(path);

        let uri = http::Uri::from_parts(parts).unwrap();
        let mut req = hyper::Request::builder()
            .version(http::Version::HTTP_2)
            .uri(uri.clone())
            .method("POST")
            .body(body)
            .unwrap();

        *req.version_mut() = http::Version::HTTP_2;
        req.headers_mut()
            .insert("method", HeaderValue::from_static("POST"));
        req.headers_mut().insert(
            "scheme",
            HeaderValue::from_str(uri.scheme_str().unwrap()).unwrap(),
        );
        req.headers_mut()
            .insert("path", HeaderValue::from_str(uri.path()).unwrap());
        req.headers_mut().insert(
            "authority",
            HeaderValue::from_str(uri.authority().unwrap().as_str()).unwrap(),
        );
        req.headers_mut().insert(
            "content-type",
            HeaderValue::from_static("application/grpc+json"),
        );
        req.headers_mut()
            .insert("user-agent", HeaderValue::from_static("dubbo-rust/1.0.0"));
        req.headers_mut()
            .insert("te", HeaderValue::from_static("trailers"));
        req.headers_mut().insert(
            "tri-service-version",
            HeaderValue::from_static("dubbo-rust/1.0.0"),
        );
        req.headers_mut()
            .insert("tri-service-group", HeaderValue::from_static("cluster"));
        req.headers_mut().insert(
            "tri-unit-info",
            HeaderValue::from_static("dubbo-rust/1.0.0"),
        );
        if let Some(_encoding) = self.send_compression_encoding {
            req.headers_mut()
                .insert("grpc-encoding", http::HeaderValue::from_static("gzip"));
        }
        req.headers_mut().insert(
            "grpc-accept-encoding",
            http::HeaderValue::from_static("gzip"),
        );

        // const (
        //     TripleContentType    = "application/grpc+proto"
        //     TripleUserAgent      = "grpc-go/1.35.0-dev"
        //     TripleServiceVersion = "tri-service-version"
        //     TripleAttachement    = "tri-attachment"
        //     TripleServiceGroup   = "tri-service-group"
        //     TripleRequestID      = "tri-req-id"
        //     TripleTraceID        = "tri-trace-traceid"
        //     TripleTraceRPCID     = "tri-trace-rpcid"
        //     TripleTraceProtoBin  = "tri-trace-proto-bin"
        //     TripleUnitInfo       = "tri-unit-info"
        // )
        req
    }

    pub async fn unary<C, M1, M2>(
        &self,
        req: Request<M1>,
        mut codec: C,
        path: http::uri::PathAndQuery,
    ) -> Result<Response<M2>, crate::status::Status>
    where
        C: Codec<Encode = M1, Decode = M2>,
        M1: Send + Sync + 'static,
        M2: Send + Sync + 'static,
    {
        let req = req.map(|m| stream::once(future::ready(m)));
        let body_stream = encode(
            codec.encoder(),
            req.into_inner().map(Ok),
            self.send_compression_encoding,
        )
        .into_stream();
        let body = hyper::Body::wrap_stream(body_stream);

        let req = self.map_request(path, body);
        let cli = self.inner.clone().builder();
        let response = cli.request(req).await;

        match response {
            Ok(v) => {
                let resp = v.map(|body| {
                    Decoding::new(body, codec.decoder(), self.send_compression_encoding)
                });
                let (mut parts, body) = Response::from_http(resp).into_parts();

                futures_util::pin_mut!(body);

                let message = body.try_next().await?.ok_or_else(|| {
                    crate::status::Status::new(
                        crate::status::Code::Internal,
                        "Missing response message.".to_string(),
                    )
                })?;

                if let Some(trailers) = body.trailer().await? {
                    let mut h = parts.into_headers();
                    h.extend(trailers.into_headers());
                    parts = Metadata::from_headers(h);
                }

                Ok(Response::from_parts(parts, message))
            }
            Err(err) => Err(crate::status::Status::new(
                crate::status::Code::Internal,
                err.to_string(),
            )),
        }
    }

    pub async fn bidi_streaming<C, M1, M2>(
        &mut self,
        req: impl IntoStreamingRequest<Message = M1>,
        mut codec: C,
        path: http::uri::PathAndQuery,
    ) -> Result<Response<Decoding<M2>>, crate::status::Status>
    where
        C: Codec<Encode = M1, Decode = M2>,
        M1: Send + Sync + 'static,
        M2: Send + Sync + 'static,
    {
        let req = req.into_streaming_request();
        let en = encode(
            codec.encoder(),
            req.into_inner().map(Ok),
            self.send_compression_encoding,
        )
        .into_stream();
        let body = hyper::Body::wrap_stream(en);

        let req = self.map_request(path, body);

        let cli = self.inner.clone().builder();
        let response = cli.request(req).await;

        match response {
            Ok(v) => {
                let resp = v.map(|body| {
                    Decoding::new(body, codec.decoder(), self.send_compression_encoding)
                });

                Ok(Response::from_http(resp))
            }
            Err(err) => Err(crate::status::Status::new(
                crate::status::Code::Internal,
                err.to_string(),
            )),
        }
    }

    pub async fn client_streaming<C, M1, M2>(
        &mut self,
        req: impl IntoStreamingRequest<Message = M1>,
        mut codec: C,
        path: http::uri::PathAndQuery,
    ) -> Result<Response<M2>, crate::status::Status>
    where
        C: Codec<Encode = M1, Decode = M2>,
        M1: Send + Sync + 'static,
        M2: Send + Sync + 'static,
    {
        let req = req.into_streaming_request();
        let en = encode(
            codec.encoder(),
            req.into_inner().map(Ok),
            self.send_compression_encoding,
        )
        .into_stream();
        let body = hyper::Body::wrap_stream(en);

        let req = self.map_request(path, body);
        let cli = self.inner.clone().builder();
        let response = cli.request(req).await;

        match response {
            Ok(v) => {
                let resp = v.map(|body| {
                    Decoding::new(body, codec.decoder(), self.send_compression_encoding)
                });
                let (mut parts, body) = Response::from_http(resp).into_parts();

                futures_util::pin_mut!(body);

                let message = body.try_next().await?.ok_or_else(|| {
                    crate::status::Status::new(
                        crate::status::Code::Internal,
                        "Missing response message.".to_string(),
                    )
                })?;

                if let Some(trailers) = body.trailer().await? {
                    let mut h = parts.into_headers();
                    h.extend(trailers.into_headers());
                    parts = Metadata::from_headers(h);
                }

                Ok(Response::from_parts(parts, message))
            }
            Err(err) => Err(crate::status::Status::new(
                crate::status::Code::Internal,
                err.to_string(),
            )),
        }
    }

    pub async fn server_streaming<C, M1, M2>(
        &mut self,
        req: Request<M1>,
        mut codec: C,
        path: http::uri::PathAndQuery,
    ) -> Result<Response<Decoding<M2>>, crate::status::Status>
    where
        C: Codec<Encode = M1, Decode = M2>,
        M1: Send + Sync + 'static,
        M2: Send + Sync + 'static,
    {
        let req = req.map(|m| stream::once(future::ready(m)));
        let en = encode(
            codec.encoder(),
            req.into_inner().map(Ok),
            self.send_compression_encoding,
        )
        .into_stream();
        let body = hyper::Body::wrap_stream(en);

        let req = self.map_request(path, body);

        let cli = self.inner.clone().builder();
        let response = cli.request(req).await;

        match response {
            Ok(v) => {
                let resp = v.map(|body| {
                    Decoding::new(body, codec.decoder(), self.send_compression_encoding)
                });

                Ok(Response::from_http(resp))
            }
            Err(err) => Err(crate::status::Status::new(
                crate::status::Code::Internal,
                err.to_string(),
            )),
        }
    }
}

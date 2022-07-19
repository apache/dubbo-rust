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

use futures_util::{StreamExt, TryStreamExt};
use http::HeaderValue;

use crate::codec::Codec;
use crate::invocation::{IntoStreamingRequest, Response};
use crate::server::encode::encode;
use crate::server::Streaming;

#[derive(Debug, Clone, Default)]
pub struct TripleClient {
    host: Option<http::uri::Authority>,
    inner: ConnectionPool,
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
        }
    }

    pub fn with_authority(self, host: http::uri::Authority) -> Self {
        TripleClient {
            host: Some(host),
            ..self
        }
    }
}

impl TripleClient {
    pub async fn bidi_streaming<C, M1, M2>(
        &mut self,
        req: impl IntoStreamingRequest<Message = M1>,
        mut codec: C,
        path: http::uri::PathAndQuery,
    ) -> Result<Response<Streaming<M2>>, tonic::Status>
    where
        C: Codec<Encode = M1, Decode = M2>,
        M1: Send + Sync + 'static,
        M2: Send + Sync + 'static,
    {
        let req = req.into_streaming_request();
        let en = encode(codec.encoder(), req.into_inner().map(Ok)).into_stream();
        let body = hyper::Body::wrap_stream(en);

        let mut parts = http::uri::Parts::default();
        parts.path_and_query = Some(path);
        parts.authority = self.host.clone();
        parts.scheme = Some(http::uri::Scheme::HTTP);

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
            HeaderValue::from_str(uri.clone().scheme_str().unwrap()).unwrap(),
        );
        req.headers_mut()
            .insert("path", HeaderValue::from_str(uri.clone().path()).unwrap());
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

        let cli = self.inner.clone().builder();
        let response = cli.request(req).await;

        match response {
            Ok(v) => {
                let resp = v.map(|body| Streaming::new(body, codec.decoder()));

                let (_parts, body) = resp.into_parts();
                Ok(Response::new(body))
            }
            Err(err) => {
                Err(tonic::Status::new(tonic::Code::Internal, err.to_string()))
            }
        }
    }
}

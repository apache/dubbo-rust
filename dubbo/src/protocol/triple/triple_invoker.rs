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

use dubbo_base::Url;
use http::{HeaderValue, Uri};
use std::{
    fmt::{Debug, Formatter},
    str::FromStr,
};
use tower_service::Service;

use crate::{
    invoker::clone_body::CloneBody,
    triple::transport::{self, connection::Connection},
};

pub struct TripleInvoker {
    url: Url,
    conn: Connection,
}

impl TripleInvoker {
    pub fn new(url: Url) -> TripleInvoker {
        let uri = http::Uri::from_str(&url.to_url()).unwrap();
        Self {
            url,
            conn: Connection::new().with_host(uri).build(),
        }
    }
}

impl Debug for TripleInvoker {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self.url).as_str())
    }
}

impl TripleInvoker {
    pub fn map_request(&self, req: http::Request<CloneBody>) -> http::Request<CloneBody> {
        let (parts, body) = req.into_parts();

        let path_and_query = parts.headers.get("path").unwrap().to_str().unwrap();

        let authority = self.url.clone().get_ip_port();

        let uri = Uri::builder()
            .scheme("http")
            .authority(authority)
            .path_and_query(path_and_query)
            .build()
            .unwrap();

        let mut req = hyper::Request::builder()
            .version(http::Version::HTTP_2)
            .uri(uri.clone())
            .method("POST")
            .body(body)
            .unwrap();

        // *req.version_mut() = http::Version::HTTP_2;
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
            HeaderValue::from_static("application/grpc+proto"),
        );
        req.headers_mut()
            .insert("user-agent", HeaderValue::from_static("dubbo-rust/0.1.0"));
        req.headers_mut()
            .insert("te", HeaderValue::from_static("trailers"));
        req.headers_mut().insert(
            "tri-service-version",
            HeaderValue::from_static("dubbo-rust/0.1.0"),
        );
        req.headers_mut()
            .insert("tri-service-group", HeaderValue::from_static("cluster"));
        req.headers_mut().insert(
            "tri-unit-info",
            HeaderValue::from_static("dubbo-rust/0.1.0"),
        );
        // if let Some(_encoding) = self.send_compression_encoding {

        // }

        req.headers_mut()
            .insert("grpc-encoding", http::HeaderValue::from_static("gzip"));

        req.headers_mut().insert(
            "grpc-accept-encoding",
            http::HeaderValue::from_static("gzip"),
        );

        // // const (
        // //     TripleContentType    = "application/grpc+proto"
        // //     TripleUserAgent      = "grpc-go/1.35.0-dev"
        // //     TripleServiceVersion = "tri-service-version"
        // //     TripleAttachement    = "tri-attachment"
        // //     TripleServiceGroup   = "tri-service-group"
        // //     TripleRequestID      = "tri-req-id"
        // //     TripleTraceID        = "tri-trace-traceid"
        // //     TripleTraceRPCID     = "tri-trace-rpcid"
        // //     TripleTraceProtoBin  = "tri-trace-proto-bin"
        // //     TripleUnitInfo       = "tri-unit-info"
        // // )
        req
    }
}

impl Service<http::Request<CloneBody>> for TripleInvoker {
    type Response = http::Response<crate::BoxBody>;

    type Error = crate::Error;

    type Future = crate::BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        <transport::connection::Connection as Service<http::Request<CloneBody>>>::poll_ready(
            &mut self.conn,
            cx,
        )
    }

    fn call(&mut self, req: http::Request<CloneBody>) -> Self::Future {
        let req = self.map_request(req);

        self.conn.call(req)
    }
}

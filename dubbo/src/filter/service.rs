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

use std::{future::Future, task::Poll};

use pin_project::pin_project;
use tower_service::Service;

use super::Filter;
use crate::boxed;
use crate::invocation::Metadata;
use crate::invocation::Request;
use crate::BoxBody;

pub struct FilterService<S, F> {
    inner: S,
    f: F,
}

impl<S, F> FilterService<S, F> {
    pub fn new(inner: S, f: F) -> Self
    where
        F: Filter,
    {
        Self { inner, f }
    }
}

impl<S, F, ReqBody, RespBody> Service<http::Request<ReqBody>> for FilterService<S, F>
where
    RespBody: Default + http_body::Body<Data = bytes::Bytes> + Send + 'static,
    RespBody::Error: Into<crate::Error>,
    F: Filter,
    S: Service<http::Request<ReqBody>, Response = http::Response<RespBody>>,
    S::Error: Into<crate::Error>,
{
    type Response = http::Response<BoxBody>;

    type Error = S::Error;

    type Future = ResponseFuture<S::Future>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        let uri = req.uri().clone();
        let method = req.method().clone();
        let version = req.version();
        let (parts, msg) = req.into_parts();

        let res = self.f.call(Request::from_parts(
            Metadata::from_headers(parts.headers),
            (),
        ));
        match res {
            Ok(req) => {
                let (metadata, _) = req.into_parts();
                let req = Request::from_parts(Metadata::from_headers(metadata.into_headers()), msg);
                let http_req = req.into_http(uri, method, version);

                let resp = self.inner.call(http_req);
                ResponseFuture::future(resp)
            }
            Err(err) => ResponseFuture::status(err),
        }
    }
}

#[pin_project]
#[derive(Debug)]
pub struct ResponseFuture<F> {
    #[pin]
    kind: Kind<F>,
}

impl<F> ResponseFuture<F> {
    fn status(status: crate::status::Status) -> Self {
        Self {
            kind: Kind::Status(Some(status)),
        }
    }

    fn future(future: F) -> Self {
        Self {
            kind: Kind::Future(future),
        }
    }
}

#[pin_project(project = KindProj)]
#[derive(Debug)]
enum Kind<F> {
    Future(#[pin] F),
    Status(Option<crate::status::Status>),
}

impl<F, E, B> Future for ResponseFuture<F>
where
    F: Future<Output = Result<http::Response<B>, E>>,
    E: Into<crate::Error>,
    B: Default + http_body::Body<Data = bytes::Bytes> + Send + 'static,
    B::Error: Into<crate::Error>,
{
    type Output = Result<http::Response<BoxBody>, E>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.project().kind.project() {
            KindProj::Future(future) => future
                .poll(cx)
                .map(|result| result.map(|res| res.map(boxed))),
            KindProj::Status(status) => {
                let response = status
                    .take()
                    .unwrap()
                    .to_http()
                    .map(|_| B::default())
                    .map(boxed);
                Poll::Ready(Ok(response))
            }
        }
    }
}

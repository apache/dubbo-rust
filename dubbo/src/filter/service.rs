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

use tower_service::Service;

use super::Filter;
use crate::invocation::{Metadata, Request};

#[derive(Clone)]
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

impl<S, F, ReqBody> Service<http::Request<ReqBody>> for FilterService<S, F>
where
    F: Filter,
    S: Service<http::Request<ReqBody>, Response = http::Response<crate::BoxBody>>,
    S::Error: Into<crate::Error>,
    S::Future: Send + 'static,
{
    type Response = http::Response<crate::BoxBody>;

    type Error = S::Error;

    type Future = crate::BoxFuture<Self::Response, Self::Error>;

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
                Box::pin(resp)
            }
            Err(err) => {
                let fut = async move { Ok(err.to_http()) };
                Box::pin(fut)
            }
        }
    }
}

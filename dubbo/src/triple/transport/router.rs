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

use std::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
};

use axum::Router;
use futures_core::Future;
use hyper::{Body, Request, Response};
use pin_project::pin_project;
use tower::ServiceExt;
use tower_service::Service;

use crate::BoxBody;

#[derive(Debug, Clone, Default)]
pub struct DubboRouter {
    pub router: Router,
}

impl DubboRouter {
    pub fn new() -> DubboRouter {
        Self {
            router: Router::new(),
        }
    }
}

impl DubboRouter {
    pub fn add_service<S>(mut self, name: String, service: S) -> Self
    where
        S: Service<Request<Body>, Response = Response<BoxBody>, Error = std::convert::Infallible>
            + Clone
            + Send
            + 'static,
        S::Future: Send + 'static,
    {
        let svc = service.map_response(|res| res.map(axum::body::boxed));
        // *{bubbo} represents wildcard router
        self.router = self.router.route(&format!("/{}/*dubbo", name), svc);

        self
    }
}

impl Service<Request<Body>> for DubboRouter {
    type Response = Response<BoxBody>;
    type Error = crate::Error;
    type Future = RoutesFuture;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        RoutesFuture(self.router.call(req))
    }
}

#[pin_project]
pub struct RoutesFuture(#[pin] axum::routing::future::RouteFuture<Body, std::convert::Infallible>);

impl fmt::Debug for RoutesFuture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("RoutesFuture").finish()
    }
}

impl Future for RoutesFuture {
    type Output = Result<Response<BoxBody>, crate::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match futures_util::ready!(self.project().0.poll(cx)) {
            Ok(res) => Ok(res.map(crate::boxed)).into(),
            Err(err) => match err {},
        }
    }
}

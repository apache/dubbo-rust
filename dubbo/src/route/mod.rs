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

use std::pin::Pin;

use dubbo_base::StdError;
use dubbo_logger::tracing::debug;
use futures_core::{ready, Future};
use futures_util::{future::Ready, FutureExt, TryFutureExt};
use tower::{buffer::Buffer, util::FutureService};
use tower_service::Service;

use crate::{
    codegen::{RpcInvocation, TripleInvoker},
    invoker::clone_invoker::CloneInvoker,
    param::Param,
    svc::NewService,
};

pub struct NewRoutes<N> {
    inner: N,
}

pub struct NewRoutesFuture<S, T> {
    inner: RoutesFutureInnerState<S>,
    #[allow(dead_code)]
    target: T,
}

pub enum RoutesFutureInnerState<S> {
    Service(S),
    Future(
        Pin<
            Box<
                dyn Future<Output = Result<Vec<CloneInvoker<TripleInvoker>>, StdError>>
                    + Send
                    + 'static,
            >,
        >,
    ),
    Ready(Vec<CloneInvoker<TripleInvoker>>),
}

#[derive(Clone)]
pub struct Routes<T> {
    #[allow(dead_code)]
    target: T,
    invokers: Vec<CloneInvoker<TripleInvoker>>,
}

impl<N> NewRoutes<N> {
    pub fn new(inner: N) -> Self {
        Self { inner }
    }
}

impl<N> NewRoutes<N> {
    const MAX_ROUTE_BUFFER_SIZE: usize = 16;

    pub fn layer() -> impl tower_layer::Layer<N, Service = Self> {
        tower_layer::layer_fn(|inner: N| NewRoutes::new(inner))
    }
}

impl<N, T> NewService<T> for NewRoutes<N>
where
    T: Param<RpcInvocation> + Clone + Send + Unpin + 'static,
    // NewDirectory
    N: NewService<T>,
    // Directory
    N::Service: Service<(), Response = Vec<CloneInvoker<TripleInvoker>>> + Unpin + Send + 'static,
    <N::Service as Service<()>>::Error: Into<StdError>,
    <N::Service as Service<()>>::Future: Send + 'static,
{
    type Service =
        Buffer<FutureService<NewRoutesFuture<<N as NewService<T>>::Service, T>, Routes<T>>, ()>;

    fn new_service(&self, target: T) -> Self::Service {
        let inner = self.inner.new_service(target.clone());

        Buffer::new(
            FutureService::new(NewRoutesFuture {
                inner: RoutesFutureInnerState::Service(inner),
                target,
            }),
            Self::MAX_ROUTE_BUFFER_SIZE,
        )
    }
}

impl<N, T> Future for NewRoutesFuture<N, T>
where
    T: Param<RpcInvocation> + Clone + Unpin,
    // Directory
    N: Service<(), Response = Vec<CloneInvoker<TripleInvoker>>> + Unpin,
    N::Error: Into<StdError>,
    N::Future: Send + 'static,
{
    type Output = Result<Routes<T>, StdError>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();

        loop {
            match this.inner {
                RoutesFutureInnerState::Service(ref mut service) => {
                    debug!("RoutesFutureInnerState::Service");
                    let _ = ready!(service.poll_ready(cx)).map_err(Into::into)?;
                    let fut = service.call(()).map_err(|e| e.into()).boxed();
                    this.inner = RoutesFutureInnerState::Future(fut);
                }
                RoutesFutureInnerState::Future(ref mut futures) => {
                    debug!("RoutesFutureInnerState::Future");
                    let invokers = ready!(futures.as_mut().poll(cx))?;
                    this.inner = RoutesFutureInnerState::Ready(invokers);
                }
                RoutesFutureInnerState::Ready(ref invokers) => {
                    debug!("RoutesFutureInnerState::Ready");
                    let target = this.target.clone();
                    return std::task::Poll::Ready(Ok(Routes {
                        invokers: invokers.clone(),
                        target,
                    }));
                }
            }
        }
    }
}

impl<T> Service<()> for Routes<T>
where
    T: Param<RpcInvocation> + Clone,
{
    type Response = Vec<CloneInvoker<TripleInvoker>>;

    type Error = StdError;

    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: ()) -> Self::Future {
        // some router operator
        // if new_invokers changed, send new invokers to routes_rx after router operator
        futures_util::future::ok(self.invokers.clone())
    }
}

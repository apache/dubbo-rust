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
use crate::StdError;
use futures_core::future::BoxFuture;
use tower::{discover::ServiceList, ServiceExt};
use tower_service::Service;

use crate::{
    codegen::RpcInvocation,
    invoker::{clone_body::CloneBody, clone_invoker::CloneInvoker},
    param::Param,
    svc::NewService,
};

use crate::protocol::triple::triple_invoker::TripleInvoker;

pub struct NewLoadBalancer<N> {
    inner: N,
}

#[derive(Clone)]
pub struct LoadBalancer<S> {
    inner: S, // Routes service
}

impl<N> NewLoadBalancer<N> {
    pub fn layer() -> impl tower_layer::Layer<N, Service = Self> {
        tower_layer::layer_fn(|inner| {
            NewLoadBalancer {
                inner, // NewRoutes
            }
        })
    }
}

impl<N, T> NewService<T> for NewLoadBalancer<N>
where
    T: Param<RpcInvocation> + Clone,
    // NewRoutes
    N: NewService<T>,
{
    type Service = LoadBalancer<N::Service>;

    fn new_service(&self, target: T) -> Self::Service {
        // Routes service
        let svc = self.inner.new_service(target);

        LoadBalancer { inner: svc }
    }
}

impl<N> Service<http::Request<CloneBody>> for LoadBalancer<N>
where
    // Routes service
    N: Service<(), Response = Vec<CloneInvoker<TripleInvoker>>> + Clone,
    N::Error: Into<StdError> + Send,
    N::Future: Send + 'static,
{
    type Response = <CloneInvoker<TripleInvoker> as Service<http::Request<CloneBody>>>::Response;

    type Error = StdError;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: http::Request<CloneBody>) -> Self::Future {
        let routes = self.inner.call(());

        let fut = async move {
            let routes = routes.await;

            let routes: Vec<CloneInvoker<TripleInvoker>> = match routes {
                Err(e) => return Err(Into::<StdError>::into(e)),
                Ok(routes) => routes,
            };

            let service_list: Vec<_> = routes
                .into_iter()
                .map(|invoker| tower::load::Constant::new(invoker, 1))
                .collect();

            let service_list = ServiceList::new(service_list);

            let p2c = tower::balance::p2c::Balance::new(service_list);

            p2c.oneshot(req).await
        };

        Box::pin(fut)
    }
}

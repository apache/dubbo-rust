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

pub mod random;

use crate::{invocation::Metadata, StdError};
use futures_core::future::BoxFuture;
use tower::{balance::p2c::Balance, discover::ServiceList, load::Constant, ServiceExt};
use tower_service::Service;
use tracing::debug;

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
pub struct LoadBalancerSvc<S> {
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
    type Service = LoadBalancerSvc<N::Service>;

    fn new_service(&self, target: T) -> Self::Service {
        // Routes service
        let svc = self.inner.new_service(target);

        LoadBalancerSvc { inner: svc }
    }
}

impl<N> Service<http::Request<CloneBody>> for LoadBalancerSvc<N>
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

            // let service_list: Vec<_> = routes
            //     .into_iter()
            //     // .map(|invoker| tower::load::Constant::new(invoker, 1))
            //     .collect();

            // let rdm = RandomLoadBalancer::default();
            let metadata = Metadata::from_headers(req.headers().clone());
            // let invks = rdm.select_invokers(service_list, metadata);
            // invks.oneshot(req).await
            // let service_list = ServiceList::new(service_list);

            // let p2c = tower::balance::p2c::Balance::new(service_list);
            let p = P2cBalanceer::default();
            let ivk: Balance<
                ServiceList<Vec<Constant<CloneInvoker<TripleInvoker>, i32>>>,
                http::Request<CloneBody>,
            > = p.select_invokers(routes, metadata);

            ivk.oneshot(req).await
        };

        Box::pin(fut)
    }
}

pub trait LoadBalancer {
    type Invoker;

    fn select_invokers(
        self,
        invokers: Vec<CloneInvoker<TripleInvoker>>,
        metadata: Metadata,
    ) -> Self::Invoker;
}

#[derive(Debug, Default)]
pub struct P2cBalanceer {}

impl LoadBalancer for P2cBalanceer {
    type Invoker = Balance<
        ServiceList<Vec<Constant<CloneInvoker<TripleInvoker>, i32>>>,
        http::Request<CloneBody>,
    >;

    fn select_invokers(
        self,
        invokers: Vec<CloneInvoker<TripleInvoker>>,
        _metadata: Metadata,
    ) -> Self::Invoker {
        debug!("p2c load balancer");
        let service_list: Vec<_> = invokers
            .into_iter()
            .map(|invoker| tower::load::Constant::new(invoker, 1))
            .collect();
        let service_list = ServiceList::new(service_list);

        let p = tower::balance::p2c::Balance::new(service_list);
        p
    }
}

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

use futures_core::future::BoxFuture;
use http::Request;
use tower::ServiceExt;
use tower_service::Service;

use crate::{
    codegen::RpcInvocation, invoker::clone_body::CloneBody, param::Param, svc::NewService,
};

pub struct NewCluster<N> {
    inner: N, // new loadbalancer service
    strategy: ClusterStrategy,
}

pub struct Cluster<S> {
    inner: S, // loadbalancer service
    strategy: ClusterStrategy,
}

impl<N> NewCluster<N> {
    pub fn layer(strategy: ClusterStrategy) -> impl tower_layer::Layer<N, Service = Self> {
        tower_layer::layer_fn(move |inner: N| {
            NewCluster {
                inner, // new loadbalancer service
                strategy: strategy.clone(),
            }
        })
    }
}

impl<S, T> NewService<T> for NewCluster<S>
where
    T: Param<RpcInvocation>,
    // new loadbalancer service
    S: NewService<T>,
{
    type Service = Cluster<S::Service>;

    fn new_service(&self, target: T) -> Self::Service {
        Cluster {
            inner: self.inner.new_service(target),
            strategy: self.strategy.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ClusterStrategy {
    Failfast,
    #[default]
    Failover,
}

impl ClusterStrategy {
    const DEFAULT_FAILOVER_ATTEMPTS: usize = 2;

    pub fn parse(strategy: &str) -> Option<Self> {
        match strategy {
            "failfast" => Some(Self::Failfast),
            "failover" => Some(Self::Failover),
            _ => None,
        }
    }
}

impl<S> Service<Request<hyper::Body>> for Cluster<S>
where
    S: Service<Request<CloneBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
    S::Response: Send + 'static,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<hyper::Body>) -> Self::Future {
        let (parts, body) = req.into_parts();
        let clone_body = CloneBody::new(body);
        let replay_body = clone_body.clone();
        let method = parts.method;
        let uri = parts.uri;
        let version = parts.version;
        let headers = parts.headers;
        let extensions = parts.extensions;
        let inner = self.inner.clone();
        let strategy = self.strategy.clone();

        Box::pin(async move {
            let make_request = |body, extensions| {
                let mut req = Request::new(body);
                *req.method_mut() = method.clone();
                *req.uri_mut() = uri.clone();
                *req.version_mut() = version;
                *req.headers_mut() = headers.clone();
                if let Some(extensions) = extensions {
                    *req.extensions_mut() = extensions;
                }
                req
            };

            match strategy {
                ClusterStrategy::Failfast => {
                    inner
                        .oneshot(make_request(clone_body, Some(extensions)))
                        .await
                }
                ClusterStrategy::Failover => {
                    let mut last_error = None;
                    let mut first_body = Some(clone_body);
                    let mut first_extensions = Some(extensions);
                    for _ in 0..ClusterStrategy::DEFAULT_FAILOVER_ATTEMPTS {
                        let body = first_body.take().unwrap_or_else(|| replay_body.clone());
                        let extensions = first_extensions.take();
                        match inner.clone().oneshot(make_request(body, extensions)).await {
                            Ok(response) => return Ok(response),
                            Err(err) => last_error = Some(err),
                        }
                    }

                    Err(last_error.expect("failover attempts must be greater than zero"))
                }
            }
        })
    }
}

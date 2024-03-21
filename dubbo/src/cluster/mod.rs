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

use http::Request;
use tower_service::Service;

use crate::{
    codegen::RpcInvocation, invoker::clone_body::CloneBody, param::Param, svc::NewService,
};

use self::failover::Failover;

mod failover;

pub struct NewCluster<N> {
    inner: N, // new loadbalancer service
}

pub struct Cluster<S> {
    inner: S, // failover service
}

impl<N> NewCluster<N> {
    pub fn layer() -> impl tower_layer::Layer<N, Service = Self> {
        tower_layer::layer_fn(|inner: N| {
            NewCluster {
                inner, // new loadbalancer service
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
    type Service = Cluster<Failover<S::Service>>;

    fn new_service(&self, target: T) -> Self::Service {
        Cluster {
            inner: Failover::new(self.inner.new_service(target)),
        }
    }
}

impl<S> Service<Request<hyper::Body>> for Cluster<S>
where
    S: Service<Request<CloneBody>>,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future = S::Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<hyper::Body>) -> Self::Future {
        let (parts, body) = req.into_parts();
        let clone_body = CloneBody::new(body);
        let req = Request::from_parts(parts, clone_body);
        self.inner.call(req)
    }
}

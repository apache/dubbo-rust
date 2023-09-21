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

use std::marker::PhantomData;

use dubbo_base::{svc::NewService, param::Param};
use http::Request;
use tower_service::Service;

use crate::{codegen::RpcInvocation, cluster::failover::NewFailover, StdError};

use self::{failover::Failover, clone_body::CloneBody};
 
mod clone_body; 
mod clone_invoker;
mod failover;

pub struct NewClusterService<S, Req> {
    inner: S, // new loadbalancer service
    _mark: PhantomData<Req>
}

pub struct ClusterService<S> {
    inner: S // failover service
}


impl<S, Req> NewClusterService<S, Req> 
{
 
    pub fn layer() -> impl tower_layer::Layer<S, Service = Self> {
        tower_layer::layer_fn(|inner: S| {
            NewClusterService {
                inner, // new loadbalancer service
                _mark: PhantomData
            }
        })
    }

} 

impl<S, T, Svc, Req> NewService<T> for NewClusterService<S, Req> 
where
    T: Param<RpcInvocation>, 
    // new loadbalancer service
    S: NewService<T, Service = Svc> + Clone,
    // loadbalancer service
    Svc: Service<Req> + Clone + Send,
    Svc::Error:  Into<StdError> + Send + Sync, 
    Svc::Future: Send
{ 

    type Service = ClusterService<Failover<Svc>>;

    fn new_service(&self, target: T) -> Self::Service {
        // Failover
        let inner = NewFailover::new(self.inner.clone()).new_service(target);

        ClusterService {
            inner
        }
    }
}
  
impl<S, B> Service<Request<B>> for ClusterService<S> 
where
    S: Service<Request<CloneBody<B>>>,
    B: http_body::Body + Unpin,
    B::Error: Into<StdError>,
{

    type Response = S::Response;

    type Error = S::Error;

    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }
 
    fn call(&mut self, req: Request<B>) -> Self::Future {
        let (parts, body) = req.into_parts();
        let clone_body = CloneBody::new(body);
        let req = Request::from_parts(parts, clone_body); 
        self.inner.call(req)
    }
}

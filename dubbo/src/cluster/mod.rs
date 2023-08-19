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

use std::{fmt::Debug, sync::Arc, task::Poll};

use aws_smithy_http::body::SdkBody;
use dubbo_base::Url;
use futures_util::TryFutureExt;
use tower::ready_cache::ReadyCache;
use tower_service::Service;

use crate::{
    invocation::RpcInvocation,
    protocol::{triple::triple_invoker::TripleInvoker, BoxInvoker, Invoker},
};

pub mod directory;
pub mod loadbalance;
pub mod router;

pub trait Directory: Debug {
    fn list(&self, invocation: Arc<RpcInvocation>) -> Vec<BoxInvoker>;
}

type BoxDirectory = Box<dyn Directory + Send + Sync>;

pub trait Cluster {
    fn join(&self, dir: BoxDirectory) -> BoxInvoker;
}

#[derive(Debug, Default)]
pub struct MockCluster {}

impl Cluster for MockCluster {
    fn join(&self, dir: BoxDirectory) -> BoxInvoker {
        Box::new(FailoverCluster::new(dir))
    }
}

// 在Cluster上进行缓存Service
#[derive(Debug)]
pub struct FailoverCluster {
    dir: Arc<BoxDirectory>,
    caches: ReadyCache<usize, BoxInvoker, http::Request<SdkBody>>,
}

impl FailoverCluster {
    pub fn new(dir: BoxDirectory) -> FailoverCluster {
        Self {
            dir: Arc::new(dir),
            caches: ReadyCache::default(),
        }
    }
}

impl Service<http::Request<SdkBody>> for FailoverCluster {
    type Response = http::Response<crate::BoxBody>;

    type Error = crate::Error;

    type Future = crate::BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        // if self.dir.is_empty() return err
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<SdkBody>) -> Self::Future {
        // let clone_body = req.body().try_clone().unwrap();
        // let mut clone_req = http::Request::builder()
        //     .uri(req.uri().clone())
        //     .method(req.method().clone());
        // *clone_req.headers_mut().unwrap() = req.headers().clone();
        // let r = clone_req.body(clone_body).unwrap();
        let invokers = self.dir.list(
            RpcInvocation::default()
                .with_service_unique_name("hello".to_string())
                .into(),
        );
        let mut i: usize = 0;
        for invoker in invokers {
            self.caches.push(i, invoker);
            i += 1;
        }

        Box::pin(self.caches.call_ready_index(0, req).map_err(Into::into))
    }
}

impl Invoker<http::Request<SdkBody>> for FailoverCluster {
    fn get_url(&self) -> dubbo_base::Url {
        Url::from_url("triple://127.0.0.1:8888/helloworld.Greeter").unwrap()
    }
}

#[derive(Debug, Default)]
pub struct MockDirectory {
    // router_chain: RouterChain,
}

impl MockDirectory {
    pub fn new() -> MockDirectory {
        // let router_chain = get_global_router_manager().read().unwrap().get_router_chain(invocation);
        Self {
            // router_chain
        }
    }
}

impl Directory for MockDirectory {
    fn list(&self, _invo: Arc<RpcInvocation>) -> Vec<BoxInvoker> {
        // tracing::info!("MockDirectory: {}", meta);
        let u = Url::from_url("triple://127.0.0.1:8888/helloworld.Greeter").unwrap();
        vec![Box::new(TripleInvoker::new(u))]
        // self.router_chain.route(u, invo);
    }
}

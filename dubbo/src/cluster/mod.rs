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

use std::{collections::HashMap, fmt::Debug, sync::Arc, task::Poll};

use aws_smithy_http::body::SdkBody;
use dubbo_base::Url;
use dyn_clone::DynClone;

use crate::{
    empty_body,
    invocation::RpcInvocation,
    protocol::{BoxInvoker, Invoker},
};

pub mod directory;
pub mod loadbalance;

pub trait Directory: Debug + DynClone {
    fn list(&self, invocation: Arc<RpcInvocation>) -> Vec<BoxInvoker>;
    // fn is_empty(&self) -> bool;
}

dyn_clone::clone_trait_object!(Directory);

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
#[derive(Clone, Debug)]
pub struct FailoverCluster {
    dir: Arc<BoxDirectory>,
}

impl FailoverCluster {
    pub fn new(dir: BoxDirectory) -> FailoverCluster {
        Self { dir: Arc::new(dir) }
    }
}

impl Invoker<http::Request<SdkBody>> for FailoverCluster {
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
        for mut invoker in invokers {
            let fut = async move {
                let res = invoker.call(req).await;
                return res;
            };
            return Box::pin(fut);
        }
        Box::pin(async move {
            Ok(http::Response::builder()
                .status(200)
                .header("grpc-status", "12")
                .header("content-type", "application/grpc")
                .body(empty_body())
                .unwrap())
        })
    }

    fn get_url(&self) -> dubbo_base::Url {
        Url::from_url("triple://127.0.0.1:8888/helloworld.Greeter").unwrap()
    }
}

#[derive(Debug, Default, Clone)]
pub struct MockDirectory {
    // router_chain: RouterChain,
    invokers: Vec<BoxInvoker>,
}

impl MockDirectory {
    pub fn new(invokers: Vec<BoxInvoker>) -> MockDirectory {
        Self {
            // router_chain: RouterChain::default(),
            invokers,
        }
    }
}

impl Directory for MockDirectory {
    fn list(&self, _invo: Arc<RpcInvocation>) -> Vec<BoxInvoker> {
        // tracing::info!("MockDirectory: {}", meta);
        let _u = Url::from_url("triple://127.0.0.1:8888/helloworld.Greeter").unwrap();
        // vec![Box::new(TripleInvoker::new(u))]
        // self.router_chain.route(u, invo);
        self.invokers.clone()
    }

    // fn is_empty(&self) -> bool {
    //     false
    // }
}

#[derive(Debug, Default)]
pub struct RouterChain {
    router: HashMap<String, BoxRouter>,
    invokers: Vec<BoxInvoker>,
}

impl RouterChain {
    pub fn route(&self, url: Url, invo: Arc<RpcInvocation>) -> Vec<BoxInvoker> {
        let r = self.router.get("mock").unwrap();
        r.route(self.invokers.clone(), url, invo)
    }
}

pub trait Router: Debug {
    fn route(
        &self,
        invokers: Vec<BoxInvoker>,
        url: Url,
        invo: Arc<RpcInvocation>,
    ) -> Vec<BoxInvoker>;
}

pub type BoxRouter = Box<dyn Router + Sync + Send>;

#[derive(Debug, Default)]
pub struct MockRouter {}

impl Router for MockRouter {
    fn route(
        &self,
        invokers: Vec<BoxInvoker>,
        _url: Url,
        _invo: Arc<RpcInvocation>,
    ) -> Vec<BoxInvoker> {
        invokers
    }
}

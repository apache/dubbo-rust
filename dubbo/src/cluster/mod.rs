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

use std::{fmt::Debug, sync::Arc};

use dubbo_base::Url;
use thiserror::Error;
use tower::{ready_cache::ReadyCache, ServiceExt};
use tower_service::Service;

use crate::{
    codegen::RpcInvocation,
    invocation::Invocation,
    protocol::{triple::triple_invoker::TripleInvoker, BoxInvoker, Invoker},
    triple::client::replay::ClonedBody,
    StdError,
};

pub mod directory;
pub mod loadbalance;

pub trait Directory: Debug {
    fn list(&self, service_name: String) -> Vec<BoxInvoker>;
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
    caches: ReadyCache<usize, BoxInvoker, http::Request<ClonedBody>>,
}

impl FailoverCluster {
    pub fn new(dir: BoxDirectory) -> FailoverCluster {
        Self {
            dir: Arc::new(dir),
            caches: ReadyCache::default(),
        }
    }
}

#[derive(Error, Debug)]
#[error("no available service for {0}")]
pub struct NoAvailableServiceErr(String);

#[derive(Error, Debug)]
#[error("invalid service name {0}")]
pub struct InvalidServiceNameErr(String);

impl FailoverCluster {
    async fn invoke(
        req: http::Request<ClonedBody>,
        mut invoker: BoxInvoker,
    ) -> Result<http::Response<crate::BoxBody>, (StdError, http::Request<ClonedBody>)> {
        let clone_request = FailoverCluster::clone_request(&req);
        let invoker = invoker
            .ready()
            .await
            .map_err(|e| (e, FailoverCluster::clone_request(&req)))?;
        let ret = invoker.call(req).await.map_err(|e| (e, clone_request))?;

        Ok(ret)
    }

    fn clone_request(req: &http::Request<ClonedBody>) -> http::Request<ClonedBody> {
        let mut clone = http::Request::new(req.body().clone());
        *clone.method_mut() = req.method().clone();
        *clone.uri_mut() = req.uri().clone();
        *clone.headers_mut() = req.headers().clone();
        *clone.version_mut() = req.version();

        if let Some(inv) = req.extensions().get::<RpcInvocation>().cloned() {
            clone.extensions_mut().insert(inv);
        }

        clone
    }
}

impl Service<http::Request<ClonedBody>> for FailoverCluster {
    type Response = http::Response<crate::BoxBody>;

    type Error = crate::Error;

    type Future = crate::BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.caches.poll_pending(cx).map_err(|e| e.into())
    }

    fn call(&mut self, req: http::Request<ClonedBody>) -> Self::Future {
        let inv = req.extensions().get::<RpcInvocation>();
        if inv.is_none() {
            return Box::pin(async move {
                return Err(
                    InvalidServiceNameErr("service name must not be null".to_owned()).into(),
                );
            });
        }

        let inv = inv.unwrap();
        let service_name = inv.get_target_service_unique_name();

        let invokers = self.dir.list(service_name.clone());

        Box::pin(async move {
            let mut current_req = req;
            let mut last_err = None;

            let is_empty = invokers.is_empty();
            if is_empty {
                return Err(NoAvailableServiceErr(service_name).into());
            }

            for invoker in invokers {
                match FailoverCluster::invoke(current_req, invoker).await {
                    Ok(resp) => return Ok(resp),
                    Err((e, cloned_request)) => {
                        current_req = cloned_request;
                        last_err = Some(e);
                    }
                }
            }

            if last_err.is_none() {
                return Err(NoAvailableServiceErr(service_name).into());
            }

            return Err(last_err.unwrap());
        })
    }
}

impl Invoker<http::Request<ClonedBody>> for FailoverCluster {
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
        Self {
            // router_chain: RouterChain::default(),
        }
    }
}

impl Directory for MockDirectory {
    fn list(&self, service_name: String) -> Vec<BoxInvoker> {
        // tracing::info!("MockDirectory: {}", meta);
        let u = Url::from_url("triple://127.0.0.1:8888/helloworld.Greeter").unwrap();
        vec![Box::new(TripleInvoker::new(u))]
        // self.router_chain.route(u, invo);
    }
}

// #[derive(Debug, Default)]
// pub struct RouterChain {
//     router: HashMap<String, BoxRouter>,
//     invokers: Arc<Vec<BoxInvoker>>,
// }

// impl RouterChain {
//     pub fn route(&mut self, url: Url, invo: Arc<RpcInvocation>) -> Arc<Vec<BoxInvoker>> {
//         let r = self.router.get("mock").unwrap();
//         r.route(self.invokers.clone(), url, invo)
//     }
// }

// pub trait Router: Debug {
//     fn route(
//         &self,
//         invokers: Arc<Vec<BoxInvoker>>,
//         url: Url,
//         invo: Arc<RpcInvocation>,
//     ) -> Arc<Vec<BoxInvoker>>;
// }

// pub type BoxRouter = Box<dyn Router + Sync + Send>;

#[cfg(test)]
pub mod tests {
    use std::task::Poll;

    use bytes::{Buf, BufMut, BytesMut};
    use dubbo_base::Url;
    use futures_util::future::poll_fn;
    use http::StatusCode;
    use http_body::Body;
    use thiserror::Error;
    use tower::ServiceExt;
    use tower_service::Service;

    use crate::{
        boxed,
        cluster::FailoverCluster,
        codegen::{Invoker, RpcInvocation},
        empty_body,
        invocation::Invocation,
        triple::client::replay::ClonedBody,
    };

    use super::Directory;

    #[derive(Error, Debug)]
    #[error("{0}")]
    struct NoResponseErr(String);

    #[derive(Debug)]
    struct MockDirectory;

    impl Directory for MockDirectory {
        fn list(&self, service_name: String) -> Vec<crate::protocol::BoxInvoker> {
            println!("get invoker list for {}", service_name);

            vec![
                Box::new(MockInvoker(1)),
                Box::new(MockInvoker(2)),
                Box::new(MockInvoker(3)),
                Box::new(MockInvoker(4)),
                Box::new(MockInvoker(5)),
            ]
        }
    }

    #[derive(Debug)]
    struct MockInvoker(u8);

    impl Invoker<http::Request<ClonedBody>> for MockInvoker {
        fn get_url(&self) -> dubbo_base::Url {
            let str = format!(
                "triple://127.0.0.1:8888/failover_cluster_service/{}",
                self.0
            );
            Url::from_url(str.as_str()).unwrap()
        }
    }

    impl Service<http::Request<ClonedBody>> for MockInvoker {
        type Response = http::Response<crate::BoxBody>;

        type Error = crate::Error;

        type Future = crate::BoxFuture<http::Response<crate::BoxBody>, crate::Error>;

        fn poll_ready(
            &mut self,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: http::Request<ClonedBody>) -> Self::Future {
            let inv = req.extensions().get::<RpcInvocation>();
            if inv.is_none() {
                return Box::pin(async move {
                    let response = http::Response::builder()
                        .status(StatusCode::OK)
                        .body(empty_body())
                        .unwrap();

                    return Ok(response);
                });
            }

            let inv = inv.unwrap();
            let method_name = inv.get_method_name();
            if method_name.eq("invoker_request") {
                return Box::pin(async move {
                    let response = http::Response::builder()
                        .status(StatusCode::OK)
                        .body(boxed("invoker response".to_owned()))
                        .unwrap();

                    return Ok(response);
                });
            }

            let self_index = self.0;
            if method_name.eq("failover_request") {
                return Box::pin(async move {
                    let body = req.into_body();
                    let mut pin_body = Box::pin(body);
                    let ret = poll_fn(|cx| pin_body.as_mut().poll_data(cx)).await;

                    if ret.is_none() {
                        #[derive(Error, Debug)]
                        #[error("{0}")]
                        struct BodyIsNoneErr(&'static str);

                        return Err(BodyIsNoneErr("body must not be null").into());
                    }

                    let ret = ret.unwrap();

                    if ret.is_err() {
                        #[derive(Error, Debug)]
                        #[error("{0}")]
                        struct BodyIsErr(&'static str);
                        return Err(BodyIsErr("body must be ok").into());
                    }

                    let mut ret = ret.unwrap();

                    let index = ret.get_u8();

                    if index == self_index {
                        let ret_msg = format!("failover cluster index: {} was invoked", self_index);
                        let response = http::Response::builder()
                            .status(StatusCode::OK)
                            .body(boxed(ret_msg))
                            .unwrap();

                        return Ok(response);
                    }

                    #[derive(Error, Debug)]
                    #[error("{0}")]
                    struct NotTargetInvoker(String);

                    let ret_msg = format!(
                        "failover cluster index: {} was invoked, but is not target invoker {}",
                        self_index, index
                    );

                    println!("{}", ret_msg);
                    return Err(NotTargetInvoker(ret_msg).into());
                });
            }

            return Box::pin(async move { return Err(NoResponseErr(method_name).into()) });
        }
    }

    #[tokio::test]
    async fn test_failover_cluster() {
        let mut cluster = FailoverCluster::new(Box::new(MockDirectory));
        let cluster = cluster.ready().await;
        assert!(cluster.is_ok());

        let cluster = cluster.unwrap();

        let empty_stream = futures::stream::empty();
        let cloned_body = ClonedBody::new(empty_stream);

        let rpc_inv = RpcInvocation::default()
            .with_service_unique_name("failover_cluster_service".to_owned())
            .with_method_name("invoker_request".to_owned());

        let req = http::Request::builder()
            .extension(rpc_inv)
            .body(cloned_body)
            .unwrap();

        let ret = cluster.call(req).await;
        assert!(ret.is_ok());

        let ret = ret.unwrap();

        assert_eq!(ret.status(), StatusCode::OK);

        let body = ret.into_body();

        let mut pin = Box::pin(body);
        let data = poll_fn(|cx| pin.as_mut().poll_data(cx)).await;
        assert!(data.is_some());
        let data = data.unwrap();
        assert!(data.is_ok());
        let data = data.unwrap();

        assert_eq!(
            String::from_utf8(data.to_vec()).unwrap(),
            "invoker response"
        )
    }

    #[tokio::test]
    async fn test_failover_request() {
        let mut cluster = FailoverCluster::new(Box::new(MockDirectory));
        let cluster = cluster.ready().await;
        assert!(cluster.is_ok());

        let cluster = cluster.unwrap();

        let once_stream = futures::stream::once(async {
            let mut mut_bytes = BytesMut::default();
            mut_bytes.put_u8(5);
            return Ok(mut_bytes.freeze());
        });
        let cloned_body = ClonedBody::new(once_stream);

        let rpc_inv = RpcInvocation::default()
            .with_service_unique_name("failover_cluster_service".to_owned())
            .with_method_name("failover_request".to_owned());

        let req = http::Request::builder()
            .extension(rpc_inv)
            .body(cloned_body)
            .unwrap();

        let ret = cluster.call(req).await;
        assert!(ret.is_ok());

        let ret = ret.unwrap();

        assert_eq!(ret.status(), StatusCode::OK);

        let body = ret.into_body();

        let mut pin = Box::pin(body);
        let data = poll_fn(|cx| pin.as_mut().poll_data(cx)).await;
        assert!(data.is_some());
        let data = data.unwrap();
        assert!(data.is_ok());
        let data = data.unwrap();

        let resp_str = String::from_utf8(data.to_vec()).unwrap();
        println!("{}", resp_str);
        assert_eq!(resp_str, "failover cluster index: 5 was invoked")
    }
}

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

use std::{sync::Arc, task::Poll};

use aws_smithy_http::body::SdkBody;
use tower_service::Service;

use crate::common::url::Url;
use crate::protocol::triple::triple_invoker::TripleInvoker;
use crate::{empty_body, protocol::BoxInvoker};

pub mod directory;

pub trait Directory {
    fn list(&self, meta: String) -> Vec<BoxInvoker>;
    fn is_empty(&self) -> bool;
}

type BoxDirectory = Box<dyn Directory>;

pub struct FailoverCluster {
    dir: Arc<BoxDirectory>,
}

impl FailoverCluster {
    pub fn new(dir: BoxDirectory) -> FailoverCluster {
        Self { dir: Arc::new(dir) }
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
        println!("req: {}", req.body().content_length().unwrap());
        let clone_body = req.body().try_clone().unwrap();
        let mut clone_req = http::Request::builder()
            .uri(req.uri().clone())
            .method(req.method().clone());
        *clone_req.headers_mut().unwrap() = req.headers().clone();
        let r = clone_req.body(clone_body).unwrap();
        let invokers = self.dir.list("service_name".to_string());
        for mut invoker in invokers {
            let fut = async move {
                let res = invoker.call(r).await;
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
}

pub struct MockDirectory {}

impl Directory for MockDirectory {
    fn list(&self, meta: String) -> Vec<BoxInvoker> {
        tracing::info!("MockDirectory: {}", meta);
        let u = Url::from_url("triple://127.0.0.1:8888/helloworld.Greeter").unwrap();
        vec![Box::new(TripleInvoker::new(u))]
    }

    fn is_empty(&self) -> bool {
        false
    }
}

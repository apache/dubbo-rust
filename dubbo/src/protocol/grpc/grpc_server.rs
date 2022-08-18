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

use std::task::Context;
use std::task::Poll;

use tonic::codegen::BoxFuture;
use tonic::transport;
use tonic::transport::NamedService;
use tower::Service;

use crate::common::url::Url;
use crate::utils::boxed_clone::BoxCloneService;

// 每个service对应一个Server
#[derive(Clone)]
pub struct GrpcServer {
    inner: transport::Server,
    name: String,
}

impl GrpcServer {
    pub fn new(name: String) -> GrpcServer {
        Self {
            inner: transport::Server::builder(),
            name,
        }
    }

    pub async fn serve(mut self, url: Url) {
        let addr = url.to_url().parse().unwrap();
        let svc = super::GRPC_SERVICES
            .read()
            .unwrap()
            .get(self.name.as_str())
            .unwrap()
            .clone();
        tracing::info!("server{:?} start...", url);
        self.inner
            .add_service(MakeSvc::new(svc))
            .serve(addr)
            .await
            .unwrap();
    }
}

struct MakeSvc<T, U, E> {
    inner: BoxCloneService<T, U, E>,
}

impl<T, U, E> MakeSvc<T, U, E> {
    pub fn new(inner: BoxCloneService<T, U, E>) -> Self {
        Self { inner }
    }
}

impl<T, U, E> NamedService for MakeSvc<T, U, E> {
    const NAME: &'static str = "helloworld.Greeter";
}

impl<T, U, E> Service<T> for MakeSvc<T, U, E> {
    type Response = U;
    type Error = E;
    type Future = BoxFuture<U, E>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), E>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: T) -> BoxFuture<U, E> {
        self.inner.call(request)
    }
}

impl<T, U, E> Clone for MakeSvc<T, U, E> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

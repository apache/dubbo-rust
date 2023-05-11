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

use std::{
    fmt::Debug,
    future::Future,
    task::{Context, Poll},
};

use async_trait::async_trait;
use aws_smithy_http::body::SdkBody;
use dyn_clone::DynClone;
use tower_service::Service;

use dubbo_base::Url;

pub mod server_desc;
pub mod triple;

#[async_trait]
pub trait Protocol {
    type Invoker;

    fn destroy(&self);
    async fn export(self, url: Url) -> BoxExporter;
    async fn refer(self, url: Url) -> Self::Invoker;
}

pub trait Exporter {
    fn unexport(&self);
}

pub trait Invoker<ReqBody>: Debug + DynClone {
    type Response;

    type Error;

    type Future: Future<Output = Result<Self::Response, Self::Error>>;

    fn get_url(&self) -> Url;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    fn call(&mut self, req: ReqBody) -> Self::Future;
}

pub type BoxExporter = Box<dyn Exporter + Send + Sync>;
pub type BoxInvoker = Box<
    dyn Invoker<
            http::Request<SdkBody>,
            Response = http::Response<crate::BoxBody>,
            Error = crate::Error,
            Future = crate::BoxFuture<http::Response<crate::BoxBody>, crate::Error>,
        > + Send
        + Sync,
>;

dyn_clone::clone_trait_object!(
    Invoker<
        http::Request<SdkBody>,
        Response = http::Response<crate::BoxBody>,
        Error = crate::Error,
        Future = crate::BoxFuture<http::Response<crate::BoxBody>, crate::Error>,
    >
);

pub struct WrapperInvoker<T>(T);

impl<T, ReqBody> Service<http::Request<ReqBody>> for WrapperInvoker<T>
where
    T: Invoker<http::Request<ReqBody>, Response = http::Response<crate::BoxBody>>,
    T::Error: Into<crate::Error>,
{
    type Response = T::Response;

    type Error = T::Error;

    type Future = T::Future;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        self.0.call(req)
    }
}

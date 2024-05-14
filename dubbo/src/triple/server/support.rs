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
    sync::Arc,
    task::{Context, Poll},
};

use crate::{
    codegen::{Request, Response, UnarySvc},
    status::{Code, Status},
    triple::triple_wrapper::{TripleRequestWrapper, TripleResponseWrapper},
    BoxBody, BoxFuture, StdError,
};
use http_body::Body;
use tower::Service;

use super::TripleServer;

pub type RpcFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>;

#[derive(Debug)]
pub struct RpcContext {
    pub version: Option<String>,
    pub class_name: String,
    pub method_name: String,
    pub args: Vec<String>,
    pub result: Result<String, Status>,
}

impl RpcContext {
    pub fn new(path: String, version: Option<String>) -> Self {
        let attr: Vec<&str> = path.split("/").collect();
        RpcContext {
            version,
            class_name: attr[1].to_string(),
            method_name: attr[2].to_string(),
            args: vec![],
            result: Err(Status::new(Code::Ok, "success".to_string())),
        }
    }
}

pub trait RpcServer: Send + Sync + 'static {
    fn invoke(&self, msg: RpcContext) -> RpcFuture<RpcContext>;
    fn get_info(&self) -> (&str, Option<&str>, Vec<String>);
}

struct _Inner<T>(Arc<T>);

#[derive(Debug)]
pub struct RpcHttp2Server<T: RpcServer> {
    inner: _Inner<T>,
}

impl<T: RpcServer> RpcHttp2Server<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: _Inner(Arc::new(inner)),
        }
    }
}

impl<T, B> Service<http::Request<B>> for RpcHttp2Server<T>
where
    T: RpcServer + 'static,
    B: Body + Send + 'static,
    B::Error: Into<StdError> + Send + 'static,
{
    type Response = http::Response<BoxBody>;
    type Error = std::convert::Infallible;
    type Future = BoxFuture<Self::Response, Self::Error>;
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, request: http::Request<B>) -> Self::Future {
        let path = request.uri().path().to_string();
        let version = request
            .headers()
            .get("tri-service-version")
            .map(|e| String::from_utf8_lossy(e.as_bytes()).to_string());
        let rpc_msg = RpcContext::new(path, version);
        let rpc_unary_server = RpcUnaryServer {
            inner: self.inner.clone(),
            msg: Some(rpc_msg),
        };
        let mut server = TripleServer::new();
        let fut = async move {
            let res = server.unary(rpc_unary_server, request).await;
            Ok(res)
        };
        Box::pin(fut)
    }
}

#[allow(non_camel_case_types)]
struct RpcUnaryServer<T: RpcServer + 'static> {
    inner: _Inner<T>,
    msg: Option<RpcContext>,
}

impl<T: RpcServer> UnarySvc<TripleRequestWrapper> for RpcUnaryServer<T> {
    type Response = TripleResponseWrapper;
    type Future = BoxFuture<Response<Self::Response>, crate::status::Status>;
    fn call(&mut self, request: Request<TripleRequestWrapper>) -> Self::Future {
        let inner = self.inner.0.clone();
        let mut msg = self.msg.take().unwrap();
        msg.args = request.message.get_args();
        let fut = async move {
            let result = inner.invoke(msg).await.result;
            match result {
                Ok(res) => Ok(Response::new(TripleResponseWrapper::new(res))),
                Err(err) => Err(err),
            }
        };
        Box::pin(fut)
    }
}

impl<T: RpcServer> Clone for RpcHttp2Server<T> {
    fn clone(&self) -> Self {
        let inner = self.inner.clone();
        Self { inner }
    }
}

impl<T: RpcServer> Clone for _Inner<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

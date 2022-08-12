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

/// The request message containing the user's name.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HelloRequest {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
/// The response message containing the greetings
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HelloReply {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
/// Generated client implementations.
pub mod greeter_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use triple::client::TripleClient;
    use triple::codec::prost::ProstCodec;
    use triple::invocation::*;

    /// The greeting service definition.
    #[derive(Debug, Clone)]
    pub struct GreeterClient {
        inner: TripleClient,
        uri: String,
    }
    // impl GreeterClient<tonic::transport::Channel> {
    //     /// Attempt to create a new client by connecting to a given endpoint.
    //     pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
    //     where
    //         D: std::convert::TryInto<tonic::transport::Endpoint>,
    //         D::Error: Into<StdError>,
    //     {
    //         let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
    //         Ok(Self::new(conn))
    //     }
    // }
    impl GreeterClient {
        pub fn new() -> Self {
            Self {
                inner: TripleClient::new(),
                uri: "".to_string(),
            }
        }
        // pub fn with_interceptor<F>(
        //     inner: T,
        //     interceptor: F,
        // ) -> GreeterClient<InterceptedService<T, F>>
        // where
        //     F: tonic::service::Interceptor,
        //     T::ResponseBody: Default,
        //     T: tonic::codegen::Service<
        //         http::Request<tonic::body::BoxBody>,
        //         Response = http::Response<
        //             <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
        //         >,
        //     >,
        //     <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
        //         Into<StdError> + Send + Sync,
        // {
        //     GreeterClient::new(InterceptedService::new(inner, interceptor))
        // }
        /// Compress requests with `gzip`.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        // #[must_use]
        // pub fn send_gzip(mut self) -> Self {
        //     self.inner = self.inner.send_gzip();
        //     self
        // }
        // /// Enable decompressing responses with `gzip`.
        // #[must_use]
        // pub fn accept_gzip(mut self) -> Self {
        //     self.inner = self.inner.accept_gzip();
        //     self
        // }
        /// Sends a greeting
        pub async fn say_hello(
            &mut self,
            request: Request<super::HelloRequest>,
        ) -> Result<Response<super::HelloReply>, tonic::Status> {
            // self.inner.ready().await.map_err(|e| {
            //     tonic::Status::new(
            //         tonic::Code::Unknown,
            //         format!("Service was not ready: {}", e.into()),
            //     )
            // })?;
            let codec = ProstCodec::<super::HelloRequest, super::HelloReply>::default();
            let path = http::uri::PathAndQuery::from_static("/helloworld.Greeter/SayHello");
            self.inner.unary(request, codec, path).await
        }
    }
}
/// Generated server implementations.
pub mod greeter_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]

    use crate::protocol::server_desc::ServiceDesc;
    use crate::protocol::triple::triple_invoker::TripleInvoker;
    use crate::protocol::DubboGrpcService;
    use crate::protocol::Invoker;
    use crate::{BoxFuture, StdError};
    use async_trait::async_trait;
    use http_body::Body;
    use std::sync::Arc;
    use std::task::Context;
    use std::task::Poll;
    use tower_service::Service;
    use triple::codec::prost::ProstCodec;
    use triple::empty_body;
    use triple::invocation::{Request, Response};
    use triple::server::server::TripleServer;
    use triple::server::service::UnarySvc;

    ///Generated trait containing gRPC methods that should be implemented for use with GreeterServer.
    #[async_trait]
    pub trait Greeter: Send + Sync + 'static {
        /// Sends a greeting
        async fn say_hello(
            &self,
            request: Request<super::HelloRequest>,
        ) -> Result<Response<super::HelloReply>, tonic::Status>;
    }
    /// The greeting service definition.
    #[derive(Debug)]
    pub struct GreeterServer<T: Greeter, I> {
        inner: _Inner<T>,
        invoker: Option<_Inner<I>>,
        // accept_compression_encodings: EnabledCompressionEncodings,
        // send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Greeter, I> GreeterServer<T, I> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                invoker: None,
                // accept_compression_encodings: Default::default(),
                // send_compression_encodings: Default::default(),
            }
        }
        // pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        // where
        //     F: tonic::service::Interceptor,
        // {
        //     InterceptedService::new(Self::new(inner), interceptor)
        // }
        // /// Enable decompressing requests with `gzip`.
        // #[must_use]
        // pub fn accept_gzip(mut self) -> Self {
        //     self.accept_compression_encodings.enable_gzip();
        //     self
        // }
        // /// Compress responses with `gzip`, if the client supports it.
        // #[must_use]
        // pub fn send_gzip(mut self) -> Self {
        //     self.send_compression_encodings.enable_gzip();
        //     self
        // }
    }
    impl<T, B, I> Service<http::Request<B>> for GreeterServer<T, I>
    where
        T: Greeter,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/helloworld.Greeter/SayHello" => {
                    #[allow(non_camel_case_types)]
                    struct SayHelloSvc<T: Greeter>(pub Arc<T>);
                    impl<T: Greeter> UnarySvc<super::HelloRequest> for SayHelloSvc<T> {
                        type Response = super::HelloReply;
                        type Future = BoxFuture<Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: Request<super::HelloRequest>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).say_hello(request).await };
                            Box::pin(fut)
                        }
                    }
                    // let accept_compression_encodings = self.accept_compression_encodings;
                    // let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SayHelloSvc(inner);
                        let codec = ProstCodec::<super::HelloReply, super::HelloRequest>::default();
                        // let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                        //     accept_compression_encodings,
                        //     send_compression_encodings,
                        // );
                        let mut grpc = TripleServer::new(codec);
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: Greeter, I: Invoker + Send + Sync + 'static> Clone for GreeterServer<T, I> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                invoker: None,
                // invoker: if let Some(v) = self.invoker.borrow_mut() {
                //     Some(v.clone())
                // } else {
                //     None
                // },
                // accept_compression_encodings: self.accept_compression_encodings,
                // send_compression_encodings: self.send_compression_encodings,
            }
        }
    }

    impl<T: Greeter> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Greeter, I: Invoker> DubboGrpcService<I> for GreeterServer<T, I> {
        fn set_proxy_impl(&mut self, invoker: I) {
            self.invoker = Some(_Inner(Arc::new(invoker)));
        }

        fn service_desc(&self) -> ServiceDesc {
            ServiceDesc::new(
                "helloworld.Greeter".to_string(),
                std::collections::HashMap::new(),
            )
        }
    }

    impl<T: Greeter, I> tonic::transport::NamedService for GreeterServer<T, I> {
        const NAME: &'static str = "helloworld.Greeter";
    }

    pub fn register_server<T: Greeter>(server: T) {
        let s = GreeterServer::<_, TripleInvoker>::new(server);
        crate::protocol::triple::TRIPLE_SERVICES
            .write()
            .unwrap()
            .insert(
                "helloworld.Greeter".to_string(),
                crate::utils::boxed_clone::BoxCloneService::new(s),
            );
    }
}

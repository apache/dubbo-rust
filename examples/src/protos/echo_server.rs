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

use async_trait::async_trait;

use dubbo::protocol::triple::triple_invoker::TripleInvoker;
use dubbo::protocol::DubboGrpcService;
use dubbo::protocol::Invoker;
use http_body::Body;
use std::fmt::Debug;
use std::sync::Arc;
use tonic::codegen::BoxFuture;
use triple::codec::serde_codec::SerdeCodec;

use serde::{Deserialize, Serialize};
use std::task::Poll;
use tower_service::Service;
use triple::invocation::{Request, Response};
use triple::server::server::TripleServer;
use triple::server::service::{ClientStreamingSvc, ServerStreamingSvc, StreamingSvc, UnarySvc};
use triple::BoxBody;

pub type StdError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HelloRequest {
    pub name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HelloReply {
    pub reply: String,
}

#[async_trait]
pub trait Echo: Send + Sync + 'static {
    async fn hello(
        &self,
        req: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, tonic::Status>;

    async fn client_streaming_echo(
        &self,
        request: Request<triple::server::Decoding<HelloRequest>>,
    ) -> Result<Response<HelloReply>, tonic::Status>;

    type BidirectionalStreamingEchoStream: futures_util::Stream<Item = Result<HelloReply, tonic::Status>>
        + Send
        + 'static;
    /// BidirectionalStreamingEcho is bidi streaming.
    async fn bidirectional_streaming_echo(
        &self,
        request: Request<triple::server::Decoding<HelloRequest>>,
    ) -> Result<Response<Self::BidirectionalStreamingEchoStream>, tonic::Status>;

    type ServerStreamingEchoStream: futures_util::Stream<Item = Result<HelloReply, tonic::Status>>
        + Send
        + 'static;
    async fn server_streaming_echo(
        &self,
        req: Request<HelloRequest>,
    ) -> Result<Response<Self::ServerStreamingEchoStream>, tonic::Status>;
}

struct _Inner<T>(Arc<T>);

pub struct EchoServer<T, I = TripleInvoker> {
    inner: _Inner<T>,
    invoker: Option<I>,
}

impl<T, I> EchoServer<T, I> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: _Inner(Arc::new(inner)),
            invoker: None,
        }
    }
}

impl<T: Echo, I, B> Service<http::Request<B>> for EchoServer<T, I>
where
    B: Body + Send + 'static,
    B::Error: Into<StdError> + Debug + Send,
    <B as Body>::Data: Send,
    I: Invoker + Send,
{
    type Response = http::Response<BoxBody>;

    type Error = std::convert::Infallible;

    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<B>) -> Self::Future {
        let inner = self.inner.clone();
        match req.uri().path() {
            "/echo/hello" => {
                struct UnaryServer<T> {
                    inner: _Inner<T>,
                }

                impl<T: Echo> UnarySvc<HelloRequest> for UnaryServer<T> {
                    type Response = HelloReply;

                    type Future = BoxFuture<Response<Self::Response>, tonic::Status>;

                    fn call(&mut self, req: Request<HelloRequest>) -> Self::Future {
                        let inner = self.inner.0.clone();
                        let fut = async move { inner.hello(req).await };
                        Box::pin(fut)
                    }
                }

                let fut = async move {
                    let mut server =
                        TripleServer::new(SerdeCodec::<HelloReply, HelloRequest>::default());
                    let resp = server.unary(UnaryServer { inner }, req).await;
                    Ok(resp)
                };

                Box::pin(fut)
            }
            "/echo/client_streaming" => {
                struct ClientStreamingServer<T> {
                    inner: _Inner<T>,
                }

                impl<T: Echo> ClientStreamingSvc<HelloRequest> for ClientStreamingServer<T> {
                    type Response = HelloReply;

                    type Future = BoxFuture<Response<Self::Response>, tonic::Status>;

                    fn call(
                        &mut self,
                        req: Request<triple::server::Decoding<HelloRequest>>,
                    ) -> Self::Future {
                        let inner = self.inner.0.clone();
                        let fut = async move { inner.client_streaming_echo(req).await };
                        Box::pin(fut)
                    }
                }

                let fut = async move {
                    let mut server =
                        TripleServer::new(SerdeCodec::<HelloReply, HelloRequest>::default());
                    let resp = server
                        .client_streaming(ClientStreamingServer { inner }, req)
                        .await;
                    Ok(resp)
                };

                Box::pin(fut)
            }
            "/echo/server_streaming" => {
                struct ServerStreamingServer<T> {
                    inner: _Inner<T>,
                }

                impl<T: Echo> ServerStreamingSvc<HelloRequest> for ServerStreamingServer<T> {
                    type Response = HelloReply;

                    type ResponseStream = T::ServerStreamingEchoStream;

                    type Future = BoxFuture<Response<Self::ResponseStream>, tonic::Status>;

                    fn call(&mut self, req: Request<HelloRequest>) -> Self::Future {
                        let inner = self.inner.0.clone();
                        let fut = async move { inner.server_streaming_echo(req).await };
                        Box::pin(fut)
                    }
                }

                let fut = async move {
                    let mut server =
                        TripleServer::new(SerdeCodec::<HelloReply, HelloRequest>::default());
                    let resp = server
                        .server_streaming(ServerStreamingServer { inner }, req)
                        .await;
                    Ok(resp)
                };

                Box::pin(fut)
            }
            "/echo/bidi_stream" => {
                struct StreamingServer<T> {
                    inner: _Inner<T>,
                }
                impl<T: Echo> StreamingSvc<HelloRequest> for StreamingServer<T> {
                    type Response = HelloReply;

                    type ResponseStream = T::BidirectionalStreamingEchoStream;

                    type Future = BoxFuture<Response<Self::ResponseStream>, tonic::Status>;

                    fn call(
                        &mut self,
                        req: Request<triple::server::Decoding<HelloRequest>>,
                    ) -> Self::Future {
                        let inner = self.inner.0.clone();
                        let fut = async move { inner.bidirectional_streaming_echo(req).await };
                        Box::pin(fut)
                    }
                }

                let fut = async move {
                    let mut server =
                        TripleServer::new(SerdeCodec::<HelloReply, HelloRequest>::default());
                    let resp = server.bidi_streaming(StreamingServer { inner }, req).await;
                    Ok(resp)
                };

                Box::pin(fut)
            }
            _ => Box::pin(async move {
                Ok(http::Response::builder()
                    .status(200)
                    .header("grpc-status", "12")
                    .header("content-type", "application/grpc")
                    .body(
                        http_body::Empty::new()
                            .map_err(|err| match err {})
                            .boxed_unsync(),
                    )
                    .unwrap())
            }),
        }
    }
}

impl<T, I> DubboGrpcService<I> for EchoServer<T, I>
where
    I: Invoker + Send + Sync + 'static,
{
    fn set_proxy_impl(&mut self, invoker: I) {
        self.invoker = Some(invoker);
    }

    fn service_desc(&self) -> dubbo::protocol::server_desc::ServiceDesc {
        todo!()
    }
}

impl<T> Clone for _Inner<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Debug> Debug for _Inner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Inner {:?}", self.0)
    }
}

impl<T: Echo, I: Invoker + Send + Sync + 'static> Clone for EchoServer<T, I> {
    fn clone(&self) -> Self {
        let inner = self.inner.clone();
        Self {
            inner,
            invoker: None,
        }
    }
}

pub fn register_server<T: Echo>(server: T) {
    let s = EchoServer::<_, TripleInvoker>::new(server);
    dubbo::protocol::triple::TRIPLE_SERVICES
        .write()
        .unwrap()
        .insert(
            "echo".to_string(),
            dubbo::utils::boxed_clone::BoxCloneService::new(s),
        );
}

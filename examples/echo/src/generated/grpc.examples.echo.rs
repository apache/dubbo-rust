/// EchoRequest is the request for echo.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EchoRequest {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
/// EchoResponse is the response for echo.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EchoResponse {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
/// Generated client implementations.
pub mod echo_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use dubbo::codegen::*;
    /// Echo is the echo service.
    #[derive(Clone)]
    pub struct EchoClient {
        inner: TripleClient,
    }
    impl EchoClient {
        pub fn connect(host: String) -> Self {
            let cli = TripleClient::connect(host);
            EchoClient { inner: cli }
        }
        pub fn new(builder: ClientBuilder) -> Self {
            Self {
                inner: TripleClient::new(builder),
            }
        }
        /// UnaryEcho is unary echo.
        pub async fn unary_echo(
            &mut self,
            request: Request<super::EchoRequest>,
        ) -> Result<Response<super::EchoResponse>, dubbo::status::Status> {
            let codec =
                dubbo::codegen::ProstCodec::<super::EchoRequest, super::EchoResponse>::default();
            let invocation = RpcInvocation::default()
                .with_service_unique_name(String::from("grpc.examples.echo.Echo"))
                .with_method_name(String::from("UnaryEcho"));
            let path = http::uri::PathAndQuery::from_static("/grpc.examples.echo.Echo/UnaryEcho");
            self.inner.unary(request, codec, path, invocation).await
        }
        /// ServerStreamingEcho is server side streaming.
        pub async fn server_streaming_echo(
            &mut self,
            request: Request<super::EchoRequest>,
        ) -> Result<Response<Decoding<super::EchoResponse>>, dubbo::status::Status> {
            let codec =
                dubbo::codegen::ProstCodec::<super::EchoRequest, super::EchoResponse>::default();
            let invocation = RpcInvocation::default()
                .with_service_unique_name(String::from("grpc.examples.echo.Echo"))
                .with_method_name(String::from("ServerStreamingEcho"));
            let path = http::uri::PathAndQuery::from_static(
                "/grpc.examples.echo.Echo/ServerStreamingEcho",
            );
            self.inner
                .server_streaming(request, codec, path, invocation)
                .await
        }
        /// ClientStreamingEcho is client side streaming.
        pub async fn client_streaming_echo(
            &mut self,
            request: impl IntoStreamingRequest<Message = super::EchoRequest>,
        ) -> Result<Response<super::EchoResponse>, dubbo::status::Status> {
            let codec =
                dubbo::codegen::ProstCodec::<super::EchoRequest, super::EchoResponse>::default();
            let invocation = RpcInvocation::default()
                .with_service_unique_name(String::from("grpc.examples.echo.Echo"))
                .with_method_name(String::from("ClientStreamingEcho"));
            let path = http::uri::PathAndQuery::from_static(
                "/grpc.examples.echo.Echo/ClientStreamingEcho",
            );
            self.inner
                .client_streaming(request, codec, path, invocation)
                .await
        }
        /// BidirectionalStreamingEcho is bidi streaming.
        pub async fn bidirectional_streaming_echo(
            &mut self,
            request: impl IntoStreamingRequest<Message = super::EchoRequest>,
        ) -> Result<Response<Decoding<super::EchoResponse>>, dubbo::status::Status> {
            let codec =
                dubbo::codegen::ProstCodec::<super::EchoRequest, super::EchoResponse>::default();
            let invocation = RpcInvocation::default()
                .with_service_unique_name(String::from("grpc.examples.echo.Echo"))
                .with_method_name(String::from("BidirectionalStreamingEcho"));
            let path = http::uri::PathAndQuery::from_static(
                "/grpc.examples.echo.Echo/BidirectionalStreamingEcho",
            );
            self.inner
                .bidi_streaming(request, codec, path, invocation)
                .await
        }
    }
}
/// Generated server implementations.
pub mod echo_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use dubbo::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with EchoServer.
    #[async_trait]
    pub trait Echo: Send + Sync + 'static {
        /// UnaryEcho is unary echo.
        async fn unary_echo(
            &self,
            request: Request<super::EchoRequest>,
        ) -> Result<Response<super::EchoResponse>, dubbo::status::Status>;
        ///Server streaming response type for the ServerStreamingEcho method.
        type ServerStreamingEchoStream: futures_util::Stream<Item = Result<super::EchoResponse, dubbo::status::Status>>
            + Send
            + 'static;
        /// ServerStreamingEcho is server side streaming.
        async fn server_streaming_echo(
            &self,
            request: Request<super::EchoRequest>,
        ) -> Result<Response<Self::ServerStreamingEchoStream>, dubbo::status::Status>;
        /// ClientStreamingEcho is client side streaming.
        async fn client_streaming_echo(
            &self,
            request: Request<Decoding<super::EchoRequest>>,
        ) -> Result<Response<super::EchoResponse>, dubbo::status::Status>;
        ///Server streaming response type for the BidirectionalStreamingEcho method.
        type BidirectionalStreamingEchoStream: futures_util::Stream<Item = Result<super::EchoResponse, dubbo::status::Status>>
            + Send
            + 'static;
        /// BidirectionalStreamingEcho is bidi streaming.
        async fn bidirectional_streaming_echo(
            &self,
            request: Request<Decoding<super::EchoRequest>>,
        ) -> Result<Response<Self::BidirectionalStreamingEchoStream>, dubbo::status::Status>;
    }
    /// Echo is the echo service.
    #[derive(Debug)]
    pub struct EchoServer<T: Echo> {
        inner: _Inner<T>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Echo> EchoServer<T> {
        pub fn new(inner: T) -> Self {
            Self {
                inner: _Inner(Arc::new(inner)),
            }
        }
        pub fn with_filter<F>(inner: T, filter: F) -> FilterService<Self, F>
        where
            F: Filter,
        {
            FilterService::new(Self::new(inner), filter)
        }
    }
    impl<T, B> Service<http::Request<B>> for EchoServer<T>
    where
        T: Echo,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/grpc.examples.echo.Echo/UnaryEcho" => {
                    #[allow(non_camel_case_types)]
                    struct UnaryEchoServer<T: Echo> {
                        inner: _Inner<T>,
                    }
                    impl<T: Echo> UnarySvc<super::EchoRequest> for UnaryEchoServer<T> {
                        type Response = super::EchoResponse;
                        type Future = BoxFuture<Response<Self::Response>, dubbo::status::Status>;
                        fn call(&mut self, request: Request<super::EchoRequest>) -> Self::Future {
                            let inner = self.inner.0.clone();
                            let fut = async move { inner.unary_echo(request).await };
                            Box::pin(fut)
                        }
                    }
                    let fut = async move {
                        let mut server = TripleServer::new(dubbo::codegen::ProstCodec::<
                            super::EchoResponse,
                            super::EchoRequest,
                        >::default());
                        let res = server.unary(UnaryEchoServer { inner }, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc.examples.echo.Echo/ServerStreamingEcho" => {
                    #[allow(non_camel_case_types)]
                    struct ServerStreamingEchoServer<T: Echo> {
                        inner: _Inner<T>,
                    }
                    impl<T: Echo> ServerStreamingSvc<super::EchoRequest> for ServerStreamingEchoServer<T> {
                        type Response = super::EchoResponse;
                        type ResponseStream = T::ServerStreamingEchoStream;
                        type Future =
                            BoxFuture<Response<Self::ResponseStream>, dubbo::status::Status>;
                        fn call(&mut self, request: Request<super::EchoRequest>) -> Self::Future {
                            let inner = self.inner.0.clone();
                            let fut = async move { inner.server_streaming_echo(request).await };
                            Box::pin(fut)
                        }
                    }
                    let fut = async move {
                        let mut server = TripleServer::new(dubbo::codegen::ProstCodec::<
                            super::EchoResponse,
                            super::EchoRequest,
                        >::default());
                        let res = server
                            .server_streaming(ServerStreamingEchoServer { inner }, req)
                            .await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc.examples.echo.Echo/ClientStreamingEcho" => {
                    #[allow(non_camel_case_types)]
                    struct ClientStreamingEchoServer<T: Echo> {
                        inner: _Inner<T>,
                    }
                    impl<T: Echo> ClientStreamingSvc<super::EchoRequest> for ClientStreamingEchoServer<T> {
                        type Response = super::EchoResponse;
                        type Future = BoxFuture<Response<Self::Response>, dubbo::status::Status>;
                        fn call(
                            &mut self,
                            request: Request<Decoding<super::EchoRequest>>,
                        ) -> Self::Future {
                            let inner = self.inner.0.clone();
                            let fut = async move { inner.client_streaming_echo(request).await };
                            Box::pin(fut)
                        }
                    }
                    let fut = async move {
                        let mut server = TripleServer::new(dubbo::codegen::ProstCodec::<
                            super::EchoResponse,
                            super::EchoRequest,
                        >::default());
                        let res = server
                            .client_streaming(ClientStreamingEchoServer { inner }, req)
                            .await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc.examples.echo.Echo/BidirectionalStreamingEcho" => {
                    #[allow(non_camel_case_types)]
                    struct BidirectionalStreamingEchoServer<T: Echo> {
                        inner: _Inner<T>,
                    }
                    impl<T: Echo> StreamingSvc<super::EchoRequest> for BidirectionalStreamingEchoServer<T> {
                        type Response = super::EchoResponse;
                        type ResponseStream = T::BidirectionalStreamingEchoStream;
                        type Future =
                            BoxFuture<Response<Self::ResponseStream>, dubbo::status::Status>;
                        fn call(
                            &mut self,
                            request: Request<Decoding<super::EchoRequest>>,
                        ) -> Self::Future {
                            let inner = self.inner.0.clone();
                            let fut =
                                async move { inner.bidirectional_streaming_echo(request).await };
                            Box::pin(fut)
                        }
                    }
                    let fut = async move {
                        let mut server = TripleServer::new(dubbo::codegen::ProstCodec::<
                            super::EchoResponse,
                            super::EchoRequest,
                        >::default());
                        let res = server
                            .bidi_streaming(BidirectionalStreamingEchoServer { inner }, req)
                            .await;
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
    impl<T: Echo> Clone for EchoServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: Echo> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    pub fn register_server<T: Echo>(server: T) {
        let s = EchoServer::new(server);
        dubbo::protocol::triple::TRIPLE_SERVICES
            .write()
            .unwrap()
            .insert(
                "grpc.examples.echo.Echo".to_string(),
                dubbo::utils::boxed_clone::BoxCloneService::new(s),
            );
    }
}

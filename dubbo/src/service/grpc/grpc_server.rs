use std::collections::HashMap;
use std::task::Context;
use std::task::Poll;

use tonic::transport;
use tower::Service;
use tonic::transport::NamedService;
use tonic::codegen::BoxFuture;


use crate::common::url::Url;
use crate::helloworld::helloworld::greeter_server::GreeterServer;
use super::grpc_invoker::GrpcInvoker;
use crate::helloworld::helloworld::greeter_server::*;
use crate::utils::boxed_clone::BoxCloneService;

pub trait DubboGrpcService<T>
{
    fn set_proxy_impl(&mut self, invoker: T);
    fn service_desc(&self) -> ServiceDesc;
}

//type ServiceDesc struct {
//     ServiceName string
//     // The pointer to the service interface. Used to check whether the user
//     // provided implementation satisfies the interface requirements.
//     HandlerType interface{}
//     Methods     []MethodDesc
//     Streams     []StreamDesc
//     Metadata    interface{}
// }

pub struct ServiceDesc {
    service_name: String,
    // methods: HashMap<String, String> // "/Greeter/hello": "unary"
}

impl  ServiceDesc {
    pub fn new(service_name: String, _methods: HashMap<String, String>) -> Self {
        Self { service_name }
    }

    pub fn get_service_name(&self) -> String {
        self.service_name.clone()
    }
}

// codegen
pub fn register_greeter_server<T: Greeter>(server: T) -> (super::GrpcBoxCloneService, super::DubboGrpcBox) {
    let hello = GreeterServer::<T, GrpcInvoker>::new(server);
    (BoxCloneService::new(hello.clone()), Box::new(hello.clone()))
}

// 每个service对应一个Server
#[derive(Clone)]
pub struct GrpcServer {
    inner: transport::Server,
    name: String,
}

impl GrpcServer
{
    pub fn new(name: String) -> GrpcServer {
        Self {
            inner: transport::Server::builder(),
            name
        }
    }

    pub async fn serve(mut self, url: Url)
    where
    {
        let addr = url.url.clone().parse().unwrap();
        let svc = super::GRPC_SERVICES.read().unwrap().get(self.name.as_str()).unwrap().clone();
        println!("server{:?} start...", url);
        self.inner.add_service(MakeSvc::new(svc)).serve(
            addr
        ).await.unwrap();
        println!("server{:?} start...", url);
    }

}

struct MakeSvc<T, U, E> {
    inner: BoxCloneService<T, U, E>
}

impl<T, U, E> MakeSvc<T, U, E> {
    pub fn new(inner: BoxCloneService<T, U, E>) -> Self {
        Self { inner}
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
        Self { inner: self.inner.clone() }
    }
}
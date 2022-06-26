pub mod grpc_protocol;
pub mod grpc_invoker;
pub mod grpc_exporter;
pub mod grpc_server;

use std::collections::HashMap;
use std::sync::RwLock;
use lazy_static::lazy_static;

use grpc_server::DubboGrpcService;
use grpc_invoker::GrpcInvoker;
use crate::helloworld::helloworld::greeter_server::Greeter;
use crate::utils::boxed_clone::BoxCloneService;
use crate::helloworld::helloworld::{HelloRequest, HelloReply};

pub type GrpcBoxCloneService = BoxCloneService<http::Request<hyper::Body>, http::Response<tonic::body::BoxBody>, std::convert::Infallible>;

pub type DubboGrpcBox = Box<dyn DubboGrpcService<GrpcInvoker>+ Send + Sync + 'static>;

lazy_static! {
    pub static ref DUBBO_GRPC_SERVICES: RwLock<HashMap<String, Box<dyn DubboGrpcService<GrpcInvoker> + Send + Sync + 'static>>> = RwLock::new(HashMap::new());
    pub static ref GRPC_SERVICES: RwLock<HashMap<String, GrpcBoxCloneService>> = RwLock::new(HashMap::new());
}

#[tokio::test]
async fn test_hello() {
    use grpc_server::register_greeter_server;
    use crate::service::protocol::Protocol;
    use crate::common::url::Url;
    
    let (svc, dubbo_svc) = register_greeter_server(MyGreeter{});
    let svc_name = dubbo_svc.service_desc().get_service_name();
    DUBBO_GRPC_SERVICES.write().unwrap().insert(svc_name.clone(), dubbo_svc);
    GRPC_SERVICES.write().unwrap().insert(svc_name.clone(), svc);

    // server start, api: 0.0.0.0:8888/helloworld.Greeter/SayHello
    let pro = grpc_protocol::GrpcProtocol::new();
    pro.export(Url{url: "[::1]:50051".to_string(), service_key: svc_name.clone()}).await;

}


#[derive(Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: tonic::Request<HelloRequest>,
    ) -> Result<tonic::Response<HelloReply>, tonic::Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(tonic::Response::new(reply))
    }
}
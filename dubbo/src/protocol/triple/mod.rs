pub mod triple_exporter;
pub mod triple_invoker;
pub mod triple_protocol;
pub mod triple_server;

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::RwLock;

use crate::utils::boxed_clone::BoxCloneService;
use triple::BoxBody;

pub type GrpcBoxCloneService =
    BoxCloneService<http::Request<hyper::Body>, http::Response<BoxBody>, std::convert::Infallible>;

lazy_static! {
    // pub static ref DUBBO_GRPC_SERVICES: RwLock<HashMap<String, Box<dyn DubboGrpcService<GrpcInvoker> + Send + Sync + 'static>>> =
    //     RwLock::new(HashMap::new());
    pub static ref TRIPLE_SERVICES: RwLock<HashMap<String, GrpcBoxCloneService>> =
        RwLock::new(HashMap::new());
}

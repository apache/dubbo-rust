use std::sync::Once;

use tonic::client::Grpc;
use tonic::transport::Channel;
use tonic::transport::Endpoint;

use crate::service::protocol::*;
use crate::service::invocation;
use crate::common::url::Url;

pub struct GrpcInvoker {
    client: Grpc<Channel>,
    url: Url,
    once: Once
}



impl GrpcInvoker {
    pub fn new(url: Url) -> GrpcInvoker {

        let endpoint = Endpoint::new(url.url.clone()).unwrap();
        let conn = endpoint.connect_lazy();
        Self {
            url,
            client: Grpc::new(conn),
            once: Once::new()
        }
    }
}

impl Invoker for GrpcInvoker {

    fn is_available(&self) -> bool {
        true
    }

    fn destroy(&self) {
        self.once.call_once(|| {
            println!("destroy...")
        })
    }

    fn get_url(&self) -> Url {
        self.url.to_owned()
    }

    // 根据req中的数据发起req，由Client发起请求，获取响应
   fn invoke<M1>(&self, req: invocation::Request<M1>) -> invocation::Response<String>
    where
        M1: Send + 'static,
    {
        let (metadata, _) = req.into_parts();

        let resp = invocation::Response::new("string");
        let (_resp_meta, msg) = resp.into_parts();
 
        invocation::Response::from_parts(metadata, msg.to_string())
    }
}

// impl<T> invocation::Request<T> {

//     pub(crate) fn to_tonic_req(self) -> tonic::Request<T> {
//         tonic::Request::new(self.message)
//     }
// }

impl Clone for GrpcInvoker {
    fn clone(&self) -> Self {
        Self { client: self.client.clone(), url: self.url.clone(), once: Once::new() }
    }
}
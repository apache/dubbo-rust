#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HelloRequest {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HelloResponse {
    #[prost(string, tag="1")]
    pub message: ::prost::alloc::string::String,
}

use async_trait::async_trait;
use tower::Service;
use hyper::{
    Body,
    Request,
    Response,
};
use futures::future::{
    BoxFuture
};
use std::{
    task::{Poll},
};

pub type DBResp<O> = Result<xds::response::ServiceResponse<O>, xds::error::DBProstError>;


#[async_trait]
pub trait Greeter {
   async fn say_hello(&self, request: HelloRequest) -> DBResp<HelloResponse>;
}

pub struct GreeterClient {
    pub rpc_client: xds::client::RpcClient 
}

impl GreeterClient {
     pub fn new(addr: String) -> GreeterClient {
        GreeterClient {
            rpc_client: xds::client::RpcClient::new(addr) 
        }
     }
}

#[async_trait]
impl Greeter for GreeterClient {
    async fn say_hello(&self, request: HelloRequest) -> DBResp<HelloResponse> {
        let path = "dubbo/greeter.Greeter/SayHello".to_owned();
        self.rpc_client.request(request, path).await
    }
}

pub struct GreeterServer<T: 'static + Greeter + Send + Sync + Clone> {
    pub inner: T
}

impl<T: 'static + Greeter + Send + Sync + Clone> GreeterServer<T> {
    pub fn new(service: T) -> GreeterServer<T> {
        GreeterServer {
            inner: service 
        }
     }
}

impl<T> Service<Request<Body>> for GreeterServer<T> 
    where T: 'static + Greeter + Send + Sync + Clone {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
    
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
    
    fn call(&mut self, req: Request<Body>) -> Self::Future {
       let inner = self.inner.clone();
       let url = req.uri().path();
       match (req.method().clone(), url) {
          (::hyper::Method::POST, "/dubbo/greeter.Greeter/SayHello") => {
              Box::pin(async move {
                  let request = xds::ServiceRequest::try_from_hyper(req).await;
                  let proto_req = request.unwrap().try_decode().unwrap();
                  let resp = inner.say_hello(proto_req.input).await.unwrap();
                  let proto_resp = resp.try_encode();
                  let hyper_resp = proto_resp.unwrap().into_hyper();
                  Ok(hyper_resp)  
               }) 
           },
          _ => {
            Box::pin(async move { 
                Ok(xds::error::DBError::new(::hyper::StatusCode::NOT_FOUND, "not_found", "Not found").to_hyper_resp()) 
            }) 
          }
       }
   }
}
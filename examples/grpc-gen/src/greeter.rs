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

pub type DBReq<I> = xds::request::ServiceRequest<I>;
pub type DBResp<O> = Result<xds::response::ServiceResponse<O>, xds::error::DBProstError>;

use async_trait::async_trait;

#[async_trait]
pub trait Greeter {
   async fn say_hello(&self, request: DBReq<HelloRequest>) -> DBResp<HelloResponse>;
}

pub struct GreeterClient {
    pub hyper_client: xds::wrapper::HyperClient 
}

impl GreeterClient {
     pub fn new(root_url: &str) -> GreeterClient {
        GreeterClient {
            hyper_client: xds::wrapper::HyperClient::new(root_url) 
        }
     }
}

#[async_trait]
impl Greeter for GreeterClient {
    async fn say_hello(&self, request: DBReq<HelloRequest>) -> DBResp<HelloResponse> {
        self.hyper_client.request("/dubbo/greeter.Greeter/SayHello", request).await
    }
}

pub struct GreeterServer<T: 'static + Greeter + Send + Sync + Clone> {
    pub hyper_server: xds::wrapper::HyperServer<T> 
}

impl<T: 'static + Greeter + Send + Sync + Clone> GreeterServer<T> {
    pub fn new(&self, service: T) -> GreeterServer<T> {
        GreeterServer {
            hyper_server: xds::wrapper::HyperServer::new(service) 
        }
     }
}

impl<T: 'static + Greeter + Send + Sync + Clone> xds::wrapper::HyperService for GreeterServer<T> {
    fn handle(&self, req: DBReq<Vec<u8>>) -> xds::BoxFutureResp<Vec<u8>> {
       use ::futures::Future;
       let trait_object_service = self.hyper_server.service.clone();
       match (req.method.clone(), req.uri.path()) {
          (::hyper::Method::POST, "/dubbo/greeter.Greeter/SayHello") => {
              Box::pin(async move { 
                  let proto_req = req.to_proto().unwrap();  
                  let resp = trait_object_service.say_hello(proto_req).await.unwrap();  
                  let proto_resp = resp.to_proto_raw();  
                  proto_resp  
               }) 
           },
          _ => {
            Box::pin(async move { 
                Ok(xds::error::DBError::new(::hyper::StatusCode::NOT_FOUND, "not_found", "Not found").to_resp_raw()) 
            }) 
          }
       }
   }
}
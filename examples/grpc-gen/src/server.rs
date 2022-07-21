use std::convert::Infallible;
use std::net::SocketAddr;
use async_trait::async_trait;
use hyper::{Server as hyper_server};
use hyper::service::{make_service_fn};

mod greeter;
use greeter::*;
use xds::ServiceResponse;


#[derive(Default, Clone)]
pub struct HelloService {}

#[async_trait]
impl Greeter for HelloService {
    async fn say_hello(&self, request: HelloRequest) -> DBResp<HelloResponse> {
        println!("{}",request.name);
        Ok(ServiceResponse::new(HelloResponse { message: "hello, dubbo rust!".into() }))
    }
}


#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8972));
    let make_service = make_service_fn(|_conn| async {
        let server = GreeterServer::new(HelloService::default());
        Ok::<_, Infallible>(server)
    });

    let server = hyper_server::bind(&addr).serve(make_service);
    println!("Listening on http://{}", &addr);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}


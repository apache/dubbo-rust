use std::net::SocketAddr;
use xds::{ server::RpcServer };


#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8972));
    let mut server = RpcServer::new(addr);
    server.start().await;
    println!("RpcServer ok");
}
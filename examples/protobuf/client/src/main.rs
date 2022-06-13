use std::collections::hash_map::HashMap;
use xds::{ client::RpcClient };

#[tokio::main]
async fn main() {
    let task = tokio::spawn(async move {
        let mut client = RpcClient::new(String::from("http://127.0.0.1:8972"));

        println!("client call begin");
        let service_path = String::from("helloworld");
        let service_method = String::from("hello");
        let metadata = HashMap::new();
        client.call(service_path, service_method, &metadata).await;

    });
    task.await.unwrap();
}
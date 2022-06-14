use std::collections::hash_map::HashMap;
use xds::{ client::RpcClient };
use pb_message::pb::person::Person;
use prost::Message;
use hyper::body::Buf;

#[tokio::main]
async fn main() {
    let task = tokio::spawn(async move {
        let mut client = RpcClient::new(String::from("http://127.0.0.1:8972"));

        let service_path = String::from("helloworld");
        let service_method = String::from("hello");
        let metadata = HashMap::new();

        let mut person = Person::default();
        person.name = "guomiwu".to_string();
        let pbData = person.encode_to_vec();

        let callResult = client.call(service_path, service_method, &metadata, pbData).await;
        let resp = callResult.unwrap();

        //  asynchronously aggregate the chunks of the body
        let body = hyper::body::aggregate(resp).await;
        let data = body.unwrap().chunk().to_vec();

        let resPerson = Person::decode(data.as_ref()).unwrap();
        println!("resPerson={:?}", resPerson.name);

    });
    task.await.unwrap();
}
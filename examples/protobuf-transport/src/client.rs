use std::collections::hash_map::HashMap;
use xds::{client::RpcClient};
use prost::Message;
use hyper::body::Buf;
use hyper::{Request, Response};

pub mod person;
use person::Person;


#[tokio::main]
async fn main() {

    let mut client = RpcClient::new(String::from("http://127.0.0.1:8975"));

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
}

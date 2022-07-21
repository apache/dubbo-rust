
mod greeter;
use greeter::*;
#[tokio::main]
async fn main() {
    let client =  GreeterClient::new("http://127.0.0.1:8972".to_owned());
    let resp = client.say_hello(HelloRequest { name: "johankoi".into()}).await.unwrap();
    let hello_resp = resp.output;
    println!("{}",hello_resp.message);
}
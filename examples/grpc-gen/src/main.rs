
mod greeter;
use greeter::*;


fn main() {

    let client =  GreeterClient::new("0.0.0.0:8080");
    let hello_req = DBReq::new(HelloRequest { name: "johankoi".into()});
    client.say_hello(hello_req);

}

use dubbo::helloworld::helloworld::greeter_client::GreeterClient;
use dubbo::helloworld::helloworld::HelloRequest;

// pub mod hello_world {
//     tonic::include_proto!("helloworld");
// }

// cargo run --bin helloworld-client

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GreeterClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.say_hello(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
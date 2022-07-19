pub mod echo_client;
pub mod echo_server;

use futures_util::Stream;
use futures_util::StreamExt;
use std::io::ErrorKind;
use std::pin::Pin;

use tokio::sync::mpsc;

use tokio_stream::wrappers::ReceiverStream;
use tonic::async_trait;

pub use self::echo_server::{Echo, EchoServer, HelloReply, HelloRequest};
use triple::invocation::*;

#[tokio::test]
async fn test_client() {
    use self::echo_client::EchoClient;
    use self::echo_server::HelloRequest;
    use futures_util::StreamExt;
    use triple::invocation::*;

    let cli = EchoClient::new().with_uri("127.0.0.1:8888".to_string());
    let resp = cli
        .say_hello(Request::new(HelloRequest {
            name: "message from client".to_string(),
        }))
        .await;
    let resp = match resp {
        Ok(resp) => resp,
        Err(err) => return println!("{:?}", err),
    };
    let (_parts, body) = resp.into_parts();
    println!("Response: {:?}", body);

    let data = vec![
        HelloRequest {
            name: "msg1 from client".to_string(),
        },
        HelloRequest {
            name: "msg2 from client".to_string(),
        },
        HelloRequest {
            name: "msg3 from client".to_string(),
        },
    ];
    let req = futures_util::stream::iter(data);

    let bidi_resp = cli.bidirectional_streaming_echo(req).await.unwrap();

    let (_parts, mut body) = bidi_resp.into_parts();
    // let trailer = body.trailer().await.unwrap();
    // println!("trailer: {:?}", trailer);
    while let Some(item) = body.next().await {
        match item {
            Ok(v) => {
                println!("reply: {:?}", v);
            }
            Err(err) => {
                println!("err: {:?}", err);
            }
        }
    }
    let trailer = body.trailer().await.unwrap();
    println!("trailer: {:?}", trailer);
}

type ResponseStream = Pin<Box<dyn Stream<Item = Result<HelloReply, tonic::Status>> + Send>>;

#[tokio::test]
async fn test_server() {
    use std::net::ToSocketAddrs;
    use tokio::time::Duration;

    let esi = EchoServer::<EchoServerImpl>::new(EchoServerImpl {
        name: "echo server impl".to_string(),
    });
    // esi.set_proxy_impl(TripleInvoker);

    let name = "echoServer".to_string();
    println!("server listening, 0.0.0.0:8888");
    triple::transport::DubboServer::new()
        .add_service(name.clone(), esi)
        .with_http2_keepalive_timeout(Duration::from_secs(60))
        .serve(
            name.clone(),
            "0.0.0.0:8888".to_socket_addrs().unwrap().next().unwrap(),
        )
        .await
        .unwrap();
    // server.add_service(esi.into());
}

#[tokio::test]
async fn test_triple_protocol() {
    use crate::common::url::Url;
    use crate::protocol::triple::triple_protocol::TripleProtocol;
    use crate::protocol::Protocol;
    use crate::utils::boxed_clone::BoxCloneService;

    // crate::init::init();

    let esi = EchoServer::<EchoServerImpl>::new(EchoServerImpl {
        name: "echo".to_string(),
    });

    crate::protocol::triple::TRIPLE_SERVICES
        .write()
        .unwrap()
        .insert("echo".to_string(), BoxCloneService::new(esi));

    println!("triple server running, url: 0.0.0.0:8888");
    let pro = TripleProtocol::new();
    pro.export(Url {
        url: "0.0.0.0:8888".to_string(),
        service_key: "echo".to_string(),
    })
    .await;
}

#[allow(dead_code)]
#[derive(Default, Clone)]
struct EchoServerImpl {
    name: String,
}

// #[async_trait]
#[async_trait]
impl Echo for EchoServerImpl {
    async fn hello(
        &self,
        req: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, tonic::Status> {
        println!("EchoServer::hello {:?}", req.message);

        Ok(Response::new(HelloReply {
            reply: "hello, dubbo-rust".to_string(),
        }))
    }

    type BidirectionalStreamingEchoStream = ResponseStream;

    async fn bidirectional_streaming_echo(
        &self,
        request: Request<triple::server::Streaming<HelloRequest>>,
    ) -> Result<Response<Self::BidirectionalStreamingEchoStream>, tonic::Status> {
        println!("EchoServer::bidirectional_streaming_echo");

        let mut in_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(128);

        // this spawn here is required if you want to handle connection error.
        // If we just map `in_stream` and write it back as `out_stream` the `out_stream`
        // will be drooped when connection error occurs and error will never be propagated
        // to mapped version of `in_stream`.
        tokio::spawn(async move {
            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(v) => {
                        // if v.name.starts_with("msg2") {
                        //     tx.send(Err(tonic::Status::internal(format!("err: args is invalid, {:?}", v.name))
                        //     )).await.expect("working rx");
                        //     continue;
                        // }
                        tx.send(Ok(HelloReply {
                            reply: format!("server reply: {:?}", v.name),
                        }))
                        .await
                        .expect("working rx")
                    }
                    Err(err) => {
                        if let Some(io_err) = match_for_io_error(&err) {
                            if io_err.kind() == ErrorKind::BrokenPipe {
                                // here you can handle special case when client
                                // disconnected in unexpected way
                                eprintln!("\tclient disconnected: broken pipe");
                                break;
                            }
                        }

                        match tx.send(Err(err)).await {
                            Ok(_) => (),
                            Err(_err) => break, // response was droped
                        }
                    }
                }
            }
            println!("\tstream ended");
        });

        // echo just write the same data that was received
        let out_stream = ReceiverStream::new(rx);

        Ok(Response::new(
            Box::pin(out_stream) as Self::BidirectionalStreamingEchoStream
        ))
    }
}

fn match_for_io_error(err_status: &tonic::Status) -> Option<&std::io::Error> {
    let mut err: &(dyn std::error::Error + 'static) = err_status;

    loop {
        if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
            return Some(io_err);
        }

        err = match err.source() {
            Some(err) => err,
            None => return None,
        };
    }
}

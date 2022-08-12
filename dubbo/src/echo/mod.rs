/*
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.
 * The ASF licenses this file to You under the Apache License, Version 2.0
 * (the "License"); you may not use this file except in compliance with
 * the License.  You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

pub mod echo_client;
pub mod echo_server;
pub mod helloworld;

use futures_util::Stream;
use futures_util::StreamExt;
use std::io::ErrorKind;
use std::pin::Pin;

use tokio::sync::mpsc;

use tokio_stream::wrappers::ReceiverStream;
use tonic::async_trait;

pub use self::echo_server::{Echo, EchoServer, HelloReply, HelloRequest};
use triple::invocation::*;

type ResponseStream = Pin<Box<dyn Stream<Item = Result<HelloReply, tonic::Status>> + Send>>;

#[tokio::test]
async fn test_client() {
    use self::echo_client::EchoClient;
    use self::echo_server::HelloRequest;
    use futures_util::StreamExt;
    use triple::invocation::*;

    let mut cli = EchoClient::new().with_uri("http://127.0.0.1:8888".to_string());
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
            name: "msg1 from client streaming".to_string(),
        },
        HelloRequest {
            name: "msg2 from client streaming".to_string(),
        },
        HelloRequest {
            name: "msg3 from client streaming".to_string(),
        },
    ];
    let req = futures_util::stream::iter(data);
    let resp = cli.client_streaming(req).await;
    let client_streaming_resp = match resp {
        Ok(resp) => resp,
        Err(err) => return println!("{:?}", err),
    };
    let (_parts, resp_body) = client_streaming_resp.into_parts();
    println!("client streaming, Response: {:?}", resp_body);

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

    let (parts, mut body) = bidi_resp.into_parts();
    println!("parts: {:?}", parts);
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

    let resp = cli
        .server_streaming(Request::new(HelloRequest {
            name: "server streaming req".to_string(),
        }))
        .await
        .unwrap();

    let (parts, mut body) = resp.into_parts();
    println!("parts: {:?}", parts);
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

#[tokio::test]
async fn test_triple_protocol() {
    use crate::service::services::Dubbo;

    echo_server::register_server(EchoServerImpl {
        name: "echo".to_string(),
    });
    // helloworld::greeter_server::register_server(GreeterImpl {});

    Dubbo::new().start().await;
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
        println!("EchoServer::hello {:?}", req.metadata);

        Ok(Response::new(HelloReply {
            reply: "hello, dubbo-rust".to_string(),
        }))
    }

    async fn client_streaming_echo(
        &self,
        req: Request<triple::server::Decoding<HelloRequest>>,
    ) -> Result<Response<HelloReply>, tonic::Status> {
        let mut s = req.into_inner();
        loop {
            let result = s.next().await;
            match result {
                Some(Ok(val)) => println!("result: {:?}", val),
                Some(Err(val)) => println!("err: {:?}", val),
                None => break,
            }
        }
        Ok(Response::new(HelloReply {
            reply: "hello client streaming".to_string(),
        }))
    }

    type ServerStreamingEchoStream = ResponseStream;
    async fn server_streaming_echo(
        &self,
        req: Request<HelloRequest>,
    ) -> Result<Response<Self::ServerStreamingEchoStream>, tonic::Status> {
        println!("server_streaming_echo: {:?}", req.into_inner());

        let data = vec![
            Result::<_, tonic::Status>::Ok(HelloReply {
                reply: "msg1 from server".to_string(),
            }),
            Result::<_, tonic::Status>::Ok(HelloReply {
                reply: "msg2 from server".to_string(),
            }),
            Result::<_, tonic::Status>::Ok(HelloReply {
                reply: "msg3 from server".to_string(),
            }),
        ];
        let resp = futures_util::stream::iter(data);

        Ok(Response::new(Box::pin(resp)))
    }

    type BidirectionalStreamingEchoStream = ResponseStream;

    async fn bidirectional_streaming_echo(
        &self,
        request: Request<triple::server::Decoding<HelloRequest>>,
    ) -> Result<Response<Self::BidirectionalStreamingEchoStream>, tonic::Status> {
        println!(
            "EchoServer::bidirectional_streaming_echo, grpc header: {:?}",
            request.metadata
        );

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

use helloworld::greeter_server::Greeter;

struct GreeterImpl {}

#[async_trait]
impl Greeter for GreeterImpl {
    async fn say_hello(
        &self,
        request: Request<helloworld::HelloRequest>,
    ) -> Result<Response<helloworld::HelloReply>, tonic::Status> {
        println!("greeter: req: {:?}", request.metadata);
        Ok(Response::new(helloworld::HelloReply {
            message: "hello, rust".to_string(),
        }))
    }
}

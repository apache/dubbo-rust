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

use std::io::ErrorKind;
use std::pin::Pin;

use async_trait::async_trait;
use futures_util::Stream;
use futures_util::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use config::RootConfig;
use dubbo::Dubbo;
use examples::protos::hello_echo::{
    echo_server::{register_server, Echo},
    EchoRequest, EchoResponse,
};
use triple::invocation::*;

type ResponseStream =
    Pin<Box<dyn Stream<Item = Result<EchoResponse, triple::status::Status>> + Send>>;

#[tokio::main]
async fn main() {
    register_server(EchoServerImpl {
        name: "echo".to_string(),
    });

    // Dubbo::new().start().await;
    Dubbo::new()
        .with_config({
            let mut r = RootConfig::new();
            r.test_config();
            r
        })
        .start()
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
    async fn unary_echo(
        &self,
        req: Request<EchoRequest>,
    ) -> Result<Response<EchoResponse>, triple::status::Status> {
        println!("EchoServer::hello {:?}", req.metadata);

        Ok(Response::new(EchoResponse {
            message: "hello, dubbo-rust".to_string(),
        }))
    }

    async fn client_streaming_echo(
        &self,
        req: Request<triple::server::Decoding<EchoRequest>>,
    ) -> Result<Response<EchoResponse>, triple::status::Status> {
        let mut s = req.into_inner();
        loop {
            let result = s.next().await;
            match result {
                Some(Ok(val)) => println!("result: {:?}", val),
                Some(Err(val)) => println!("err: {:?}", val),
                None => break,
            }
        }
        Ok(Response::new(EchoResponse {
            message: "hello client streaming".to_string(),
        }))
    }

    type ServerStreamingEchoStream = ResponseStream;
    async fn server_streaming_echo(
        &self,
        req: Request<EchoRequest>,
    ) -> Result<Response<Self::ServerStreamingEchoStream>, triple::status::Status> {
        println!("server_streaming_echo: {:?}", req.into_inner());

        let data = vec![
            Result::<_, triple::status::Status>::Ok(EchoResponse {
                message: "msg1 from server".to_string(),
            }),
            Result::<_, triple::status::Status>::Ok(EchoResponse {
                message: "msg2 from server".to_string(),
            }),
            Result::<_, triple::status::Status>::Ok(EchoResponse {
                message: "msg3 from server".to_string(),
            }),
        ];
        let resp = futures_util::stream::iter(data);

        Ok(Response::new(Box::pin(resp)))
    }

    type BidirectionalStreamingEchoStream = ResponseStream;

    async fn bidirectional_streaming_echo(
        &self,
        request: Request<triple::server::Decoding<EchoRequest>>,
    ) -> Result<Response<Self::BidirectionalStreamingEchoStream>, triple::status::Status> {
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
                        //     tx.send(Err(triple::status::Status::internal(format!("err: args is invalid, {:?}", v.name))
                        //     )).await.expect("working rx");
                        //     continue;
                        // }
                        tx.send(Ok(EchoResponse {
                            message: format!("server reply: {:?}", v.message),
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

fn match_for_io_error(err_status: &triple::status::Status) -> Option<&std::io::Error> {
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

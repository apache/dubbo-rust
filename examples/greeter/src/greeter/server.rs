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

use std::{io::ErrorKind, pin::Pin};

use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use dubbo::{codegen::*, Dubbo};
use dubbo_config::RootConfig;
use dubbo_logger::{
    tracing::{info, span},
    Level,
};
use protos::{
    greeter_server::{register_server, Greeter},
    GreeterReply, GreeterRequest,
};
use registry_zookeeper::ZookeeperRegistry;

pub mod protos {
    #![allow(non_camel_case_types)]
    include!(concat!(env!("OUT_DIR"), "/org.apache.dubbo.sample.tri.rs"));
}

type ResponseStream =
    Pin<Box<dyn Stream<Item = Result<GreeterReply, dubbo::status::Status>> + Send>>;

#[tokio::main]
async fn main() {
    dubbo_logger::init();
    let span = span!(Level::DEBUG, "greeter.server");
    let _enter = span.enter();
    register_server(GreeterServerImpl {
        name: "greeter".to_string(),
    });
    let zkr = ZookeeperRegistry::default();
    let r = RootConfig::new();
    let r = match r.load() {
        Ok(config) => config,
        Err(_err) => panic!("err: {:?}", _err), // response was droped
    };
    let mut f = Dubbo::new()
        .with_config(r)
        .add_registry("zookeeper", Box::new(zkr));
    f.start().await;
}

#[allow(dead_code)]
#[derive(Default, Clone)]
struct GreeterServerImpl {
    name: String,
}

// #[async_trait]
#[async_trait]
impl Greeter for GreeterServerImpl {
    async fn greet(
        &self,
        request: Request<GreeterRequest>,
    ) -> Result<Response<GreeterReply>, dubbo::status::Status> {
        info!("GreeterServer::greet {:?}", request.metadata);
        println!("{:?}", request.into_inner());
        Ok(Response::new(GreeterReply {
            message: "hello, dubbo-rust".to_string(),
        }))
    }

    async fn greet_client_stream(
        &self,
        request: Request<Decoding<GreeterRequest>>,
    ) -> Result<Response<GreeterReply>, dubbo::status::Status> {
        let mut s = request.into_inner();
        loop {
            let result = s.next().await;
            match result {
                Some(Ok(val)) => println!("result: {:?}", val),
                Some(Err(val)) => println!("err: {:?}", val),
                None => break,
            }
        }
        Ok(Response::new(GreeterReply {
            message: "hello client streaming".to_string(),
        }))
    }

    type greetServerStreamStream = ResponseStream;
    async fn greet_server_stream(
        &self,
        request: Request<GreeterRequest>,
    ) -> Result<Response<Self::greetServerStreamStream>, dubbo::status::Status> {
        println!("greet_server_stream: {:?}", request.into_inner());

        let data = vec![
            Result::<_, dubbo::status::Status>::Ok(GreeterReply {
                message: "msg1 from server".to_string(),
            }),
            Result::<_, dubbo::status::Status>::Ok(GreeterReply {
                message: "msg2 from server".to_string(),
            }),
            Result::<_, dubbo::status::Status>::Ok(GreeterReply {
                message: "msg3 from server".to_string(),
            }),
        ];
        let resp = futures_util::stream::iter(data);

        Ok(Response::new(Box::pin(resp)))
    }

    type greetStreamStream = ResponseStream;
    async fn greet_stream(
        &self,
        request: Request<Decoding<GreeterRequest>>,
    ) -> Result<Response<Self::greetStreamStream>, dubbo::status::Status> {
        println!(
            "GreeterServer::greet_stream, grpc header: {:?}",
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
                        //     tx.send(Err(dubbo::status::Status::internal(format!("err: args is invalid, {:?}", v.name))
                        //     )).await.expect("working rx");
                        //     continue;
                        // }
                        tx.send(Ok(GreeterReply {
                            message: format!("server reply: {:?}", v.name),
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
            Box::pin(out_stream) as Self::greetStreamStream
        ))
    }
}

fn match_for_io_error(err_status: &dubbo::status::Status) -> Option<&std::io::Error> {
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

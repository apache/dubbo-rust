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

use core::time;
use std::future;
use std::future::ready;
use std::io::ErrorKind;
use std::pin::Pin;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use async_trait::async_trait;
use dubbo::context::Context;
use dubbo::context::RpcContext;
use dubbo::status::Code;
use dubbo::status::Status;
use futures_util::Stream;
use futures_util::StreamExt;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use dubbo::codegen::*;
use dubbo::Dubbo;
use dubbo_config::RootConfig;
use example_echo::protos::hello_echo::{
    echo_server::{register_server, Echo, EchoServer},
    EchoRequest, EchoResponse,
};

type ResponseStream =
    Pin<Box<dyn Stream<Item = Result<EchoResponse, dubbo::status::Status>> + Send>>;

#[derive(Clone)]
pub struct FakeFilter {}

impl Filter for FakeFilter {
    fn call(&mut self, req: Request<()>) -> Result<Request<()>, dubbo::status::Status> {
        println!("server fake filter: {:?}", req.metadata);
        Ok(req)
    }
}

#[derive(Clone)]
pub struct ContextFilter {}

impl Filter for ContextFilter {
    fn call(&mut self, req: Request<()>) -> Result<Request<()>, dubbo::status::Status> {
        let headers = &mut req.metadata.into_headers();

        let timeout = headers.get("timeout-countdown");

        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let mut dead_line_in_nanos = 0_u128;

        if let Some(t) = timeout {
            let timeout: u128 = t.to_str().unwrap().parse().unwrap();
            if timeout > 0_u128 {
                dead_line_in_nanos = time + timeout * 1000000;
            }
        } else {
            // TODO default timeout
            let timeout: u128 = 1000_u128 * 1000000;
            dead_line_in_nanos = time + timeout;
        }

        println!("ContextFilter tri-timeout-deadline-in-nanos : {}", dead_line_in_nanos);
        if let Some(at) = RpcContext::get_attachments() {
            let mut attachments = at.lock().unwrap();
            attachments.insert(
                String::from("tri-timeout-deadline-in-nanos"),
                Value::from(dead_line_in_nanos.to_string()),
            );
        }

        Ok(req)
    }
}

#[derive(Clone)]
pub struct TimeoutFilter {}

/// timeout count
/// 1. ContextFilter 初始化 timeout 时间，初始化后将 tri-timeout-deadline-in-nanos 放入 context 中
/// 2. TimeoutFilter read context tri-timeout-deadline-in-nanos
/// 3. 响应时计算 tri-timeout-deadline-in-nanos - current_nanos <= 0
/// 
impl Filter for TimeoutFilter {
    fn call(&mut self, req: Request<()>) -> Result<Request<()>, dubbo::status::Status> {
        if let Some(attachments) = RpcContext::get_attachments() {
            let current_nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();

            let attachments = attachments.lock().unwrap();
            let tri_timeout_deadline_in_nanos =
                attachments.get("tri-timeout-deadline-in-nanos").unwrap();
            let tri_timeout_deadline_in_nanos: u128 = tri_timeout_deadline_in_nanos
                .as_str()
                .unwrap()
                .parse()
                .unwrap();

            println!("TimeoutFilter tri-timeout-deadline-in-nanos : {}, current-nanos:{}", tri_timeout_deadline_in_nanos, current_nanos);
            if tri_timeout_deadline_in_nanos - current_nanos <= 0 {
                return Err(Status::new(Code::DeadlineExceeded, String::from("Timeout")));
            }
        }

        Ok(req)
    }
}



#[tokio::main]
async fn main() {
    register_server(EchoServerImpl {
        name: "echo".to_string(),
    });
    let server = EchoServerImpl::default();
    let s = EchoServer::<EchoServerImpl>::with_filter(server, FakeFilter {});
    let timeout_filter = FilterService::new(s, TimeoutFilter {});
    let context_filter = FilterService::new(timeout_filter, ContextFilter {});

    dubbo::protocol::triple::TRIPLE_SERVICES
        .write()
        .unwrap()
        .insert(
            "grpc.examples.echo.Echo".to_string(),
            dubbo::utils::boxed_clone::BoxCloneService::new(context_filter),
        );

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
    ) -> Result<Response<EchoResponse>, dubbo::status::Status> {
        println!("EchoServer::hello {:?}", req.metadata);

        // thread::sleep(time::Duration::from_millis(2000));

        Ok(Response::new(EchoResponse {
            message: "hello, dubbo-rust".to_string(),
        }))

    }

    async fn client_streaming_echo(
        &self,
        req: Request<Decoding<EchoRequest>>,
    ) -> Result<Response<EchoResponse>, dubbo::status::Status> {
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
    ) -> Result<Response<Self::ServerStreamingEchoStream>, dubbo::status::Status> {
        println!("server_streaming_echo: {:?}", req.into_inner());

        let data = vec![
            Result::<_, dubbo::status::Status>::Ok(EchoResponse {
                message: "msg1 from server".to_string(),
            }),
            Result::<_, dubbo::status::Status>::Ok(EchoResponse {
                message: "msg2 from server".to_string(),
            }),
            Result::<_, dubbo::status::Status>::Ok(EchoResponse {
                message: "msg3 from server".to_string(),
            }),
        ];
        let resp = futures_util::stream::iter(data);

        Ok(Response::new(Box::pin(resp)))
    }

    type BidirectionalStreamingEchoStream = ResponseStream;

    async fn bidirectional_streaming_echo(
        &self,
        request: Request<Decoding<EchoRequest>>,
    ) -> Result<Response<Self::BidirectionalStreamingEchoStream>, dubbo::status::Status> {
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
                        //     tx.send(Err(dubbo::status::Status::internal(format!("err: args is invalid, {:?}", v.name))
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

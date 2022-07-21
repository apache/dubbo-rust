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

use std::convert::Infallible;
use std::task::Poll;
use hyper::{Body, Request, Response, Server as hyper_server};
use hyper::service::{ make_service_fn, service_fn };
use tower::Service;
use futures::future::{ ready, BoxFuture };
use std::net::SocketAddr;
use crate::protocol::message::*;

pub struct RpcServer {
    pub addr: SocketAddr
}

impl RpcServer {
    pub fn new(addr: SocketAddr) -> Self {
        RpcServer {
            addr
        }
    }

    pub async fn start(&self) {
        env_logger::init();

        let make_service = make_service_fn(|_conn| async {
            let svc = RPCHandler;
            Ok::<_, Infallible>(svc)
        });

        let server = hyper_server::bind(&self.addr).serve(make_service);
        println!("Listening on http://{}", self.addr);
        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    }
}


#[derive(Copy, Clone)]
struct RPCHandler;
use std::result::Result as StdResult;

impl Service<Request<Body>> for RPCHandler {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<'static, StdResult<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<StdResult<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        Box::pin(async move {
            let whole_body = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let mut msg = Message::new();
            let mut data = &whole_body[..];

            let mut resp = ready(Ok(Response::new(Body::from("hello world with tower service! \n")))).await;
            match msg.decode(&mut data) {
                Ok(()) => {
                    let service_path = &msg.service_path;
                    let service_method = &msg.service_method;
                    let key = format!("{}.{}", service_path, service_method);
                    println!("{:?}", msg.payload);
                    println!("recieved request success, and body message decode key is: {:?}", key);
                    resp = ready(Ok(Response::new(Body::from(msg.payload)))).await;
                }
                Err(err) => {
                    eprintln!("failed to read: {}", err.to_string());
                }
            }
            resp
        })
    }
}

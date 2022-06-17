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

use dubbo_rust_protocol::jsonrpc::server::{JsonRpcServer, JsonRpcService};
use log::info;
use std::{net::SocketAddr, str::FromStr};

mod addservice;

use addservice::{add_service::AddServer, AddReq, AddResp, AddService, StdError};

#[derive(Clone)]
struct MyAdd;

#[async_trait::async_trait]
impl AddService for MyAdd {
    async fn add(&mut self, req: AddReq) -> Result<AddResp, StdError> {
        info!("get request {:?}", req);
        Ok(req.numbers.iter().sum())
    }
}

#[tokio::main]
async fn main() {
    // log
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let addr = SocketAddr::from_str("0.0.0.0:40021").unwrap();
    let rt = tokio::runtime::Handle::current();

    let clone_service = JsonRpcService::new(AddServer::new(MyAdd));

    let server = JsonRpcServer::new(&addr, rt, clone_service);

    info!("Server start at {}", addr.to_string());

    server.await.unwrap();
}

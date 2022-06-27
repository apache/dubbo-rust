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

pub mod grpc_exporter;
pub mod grpc_invoker;
pub mod grpc_protocol;
pub mod grpc_server;

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::RwLock;

use crate::helloworld::helloworld::greeter_server::Greeter;
use crate::helloworld::helloworld::{HelloReply, HelloRequest};
use crate::utils::boxed_clone::BoxCloneService;
use grpc_invoker::GrpcInvoker;
use grpc_server::DubboGrpcService;

pub type GrpcBoxCloneService = BoxCloneService<
    http::Request<hyper::Body>,
    http::Response<tonic::body::BoxBody>,
    std::convert::Infallible,
>;

pub type DubboGrpcBox = Box<dyn DubboGrpcService<GrpcInvoker> + Send + Sync + 'static>;

lazy_static! {
    pub static ref DUBBO_GRPC_SERVICES: RwLock<HashMap<String, Box<dyn DubboGrpcService<GrpcInvoker> + Send + Sync + 'static>>> =
        RwLock::new(HashMap::new());
    pub static ref GRPC_SERVICES: RwLock<HashMap<String, GrpcBoxCloneService>> =
        RwLock::new(HashMap::new());
}

#[tokio::test]
async fn test_hello() {
    use crate::common::url::Url;
    use crate::service::protocol::Protocol;
    use grpc_server::register_greeter_server;

    let (svc, dubbo_svc) = register_greeter_server(MyGreeter {});
    let svc_name = dubbo_svc.service_desc().get_service_name();
    DUBBO_GRPC_SERVICES
        .write()
        .unwrap()
        .insert(svc_name.clone(), dubbo_svc);
    GRPC_SERVICES.write().unwrap().insert(svc_name.clone(), svc);

    // server start, api: 0.0.0.0:8888/helloworld.Greeter/SayHello
    let pro = grpc_protocol::GrpcProtocol::new();
    pro.export(Url {
        url: "[::1]:50051".to_string(),
        service_key: svc_name.clone(),
    })
    .await;
}

#[derive(Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: tonic::Request<HelloRequest>,
    ) -> Result<tonic::Response<HelloReply>, tonic::Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(tonic::Response::new(reply))
    }
}

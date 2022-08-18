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

use crate::protocol::DubboGrpcService;
use crate::utils::boxed_clone::BoxCloneService;
use grpc_invoker::GrpcInvoker;

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

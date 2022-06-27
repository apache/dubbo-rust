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

use std::collections::HashMap;

use super::grpc_exporter::GrpcExporter;
use super::grpc_invoker::GrpcInvoker;
use super::grpc_server::GrpcServer;
use crate::common::url::Url;
use crate::service::protocol::Protocol;

pub struct GrpcProtocol {
    server_map: HashMap<String, GrpcServer>,
    export_map: HashMap<String, GrpcExporter<GrpcInvoker>>,
}

impl GrpcProtocol {
    pub fn new() -> Self {
        Self {
            server_map: HashMap::new(),
            export_map: HashMap::new(),
        }
    }
}

impl Default for GrpcProtocol {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Protocol for GrpcProtocol {
    type Invoker = GrpcInvoker;

    type Exporter = GrpcExporter<Self::Invoker>;

    fn destroy(&self) {
        todo!()
    }

    async fn refer(&self, url: Url) -> Self::Invoker {
        GrpcInvoker::new(url)
    }

    async fn export(self, url: Url) -> Self::Exporter {
        let service_key = url.service_key.clone();

        let exporter: GrpcExporter<GrpcInvoker> =
            GrpcExporter::new(service_key.clone(), GrpcInvoker::new(url.clone()));
        let mut export = self.export_map;
        export.insert(service_key.clone(), exporter.clone());

        // 启动服务

        let server = super::grpc_server::GrpcServer::new(service_key.clone());
        let mut server_map = self.server_map;
        server_map.insert(service_key.clone(), server.clone());
        server.serve(url.clone()).await;
        exporter
    }
}

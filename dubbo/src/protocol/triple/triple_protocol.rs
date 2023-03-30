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

use std::{boxed::Box, collections::HashMap};

use async_trait::async_trait;
use dubbo_base::Url;

use super::{
    triple_exporter::TripleExporter, triple_invoker::TripleInvoker, triple_server::TripleServer,
};
use crate::protocol::{BoxExporter, Protocol};

#[derive(Clone)]
pub struct TripleProtocol {
    servers: HashMap<String, TripleServer>,
}

impl Default for TripleProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl TripleProtocol {
    pub fn new() -> Self {
        TripleProtocol {
            servers: HashMap::new(),
        }
    }

    pub fn get_server(&self, url: Url) -> Option<TripleServer> {
        self.servers
            .get(&url.service_key)
            .map(|data| data.to_owned())
    }
}

#[async_trait]
impl Protocol for TripleProtocol {
    type Invoker = TripleInvoker;

    fn destroy(&self) {
        todo!()
    }

    async fn export(mut self, url: Url) -> BoxExporter {
        // service_key is same to key of TRIPLE_SERVICES
        let server = TripleServer::new();
        self.servers.insert(url.service_key.clone(), server.clone());
        server.serve(url.short_url().as_str().into()).await;

        Box::new(TripleExporter::new())
    }

    async fn refer(self, _url: Url) -> Self::Invoker {
        todo!()
        // TripleInvoker::new(url)
        // Self::Invoker
    }
}

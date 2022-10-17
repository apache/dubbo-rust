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

use std::boxed::Box;
use std::collections::HashMap;

use async_trait::async_trait;

use super::triple_exporter::TripleExporter;
use super::triple_invoker::TripleInvoker;
use super::triple_server::TripleServer;
use crate::common::url::Url;
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
            .get(&url.service_key.join(","))
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
        let server = TripleServer::new(url.service_key.clone());
        self.servers
            .insert(url.service_key.join(","), server.clone());
        server.serve(url.to_url()).await;

        Box::new(TripleExporter::new())
    }

    async fn refer(self, url: Url) -> Self::Invoker {
        TripleInvoker::new(url)
        // Self::Invoker
    }
}

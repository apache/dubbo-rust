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
use std::sync::{Arc, RwLock};

use super::memory_registry::MemoryRegistry;
use super::BoxRegistry;
use crate::codegen::TripleInvoker;
use crate::common::consts;
use crate::common::url::Url;
use crate::protocol::triple::triple_exporter::TripleExporter;
use crate::protocol::triple::triple_protocol::TripleProtocol;
use crate::protocol::BoxExporter;
use crate::protocol::BoxInvoker;
use crate::protocol::Protocol;

#[derive(Clone, Default)]
pub struct RegistryProtocol {
    // registerAddr: Registry
    registries: Arc<RwLock<HashMap<String, BoxRegistry>>>,
    // providerUrl: Exporter
    exporters: Arc<RwLock<HashMap<String, BoxExporter>>>,
}

impl RegistryProtocol {
    pub fn new() -> Self {
        RegistryProtocol {
            registries: Arc::new(RwLock::new(HashMap::new())),
            exporters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_registry(&mut self, url: Url) -> BoxRegistry {
        let mem = MemoryRegistry::default();
        self.registries
            .write()
            .unwrap()
            .insert(url.location, Box::new(mem.clone()));

        Box::new(mem)
    }
}

#[async_trait::async_trait]
impl Protocol for RegistryProtocol {
    type Invoker = BoxInvoker;

    fn destroy(&self) {
        todo!()
    }

    async fn export(mut self, url: Url) -> BoxExporter {
        // getProviderUrl
        // getRegisterUrl
        // init Exporter based on provider_url
        // server registry based on register_url
        // start server health check
        let registry_url = get_registry_url(url.clone());
        if !registry_url.protocol.is_empty() {
            let mut reg = self.get_registry(registry_url.clone());
            reg.register(registry_url.clone()).unwrap();
        }

        match url.clone().protocol.as_str() {
            "triple" => {
                let pro = Box::new(TripleProtocol::new());
                return pro.export(url).await;
            }
            _ => {
                tracing::error!("protocol {:?} not implemented", url.protocol);
            }
        }
        Box::new(TripleExporter::new())
    }
    async fn refer(self, url: Url) -> Self::Invoker {
        // getRegisterUrl
        // get Registry from registry_url
        // init directory based on registry_url and Registry
        // init Cluster based on Directory generates Invoker
        Box::new(TripleInvoker::new(url))
    }
}

fn get_registry_url(mut url: Url) -> Url {
    if url.protocol == consts::REGISTRY_PROTOCOL {
        url.protocol = url.get_param("registry".to_string()).unwrap();
    }

    url
}

fn get_provider_url(url: Url) -> Url {
    url
}

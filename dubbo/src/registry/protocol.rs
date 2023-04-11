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

use dubbo_base::Url;
use dubbo_logger::tracing;
use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
    sync::{Arc, Mutex, RwLock},
};

use super::{memory_registry::MemoryRegistry, BoxRegistry};
use crate::{
    protocol::{
        triple::{triple_exporter::TripleExporter, triple_protocol::TripleProtocol},
        BoxExporter, BoxInvoker, Protocol,
    },
    registry::types::Registries,
};

#[derive(Clone, Default)]
pub struct RegistryProtocol {
    // registerAddr: Registry
    registries: Option<Registries>,
    // providerUrl: Exporter
    exporters: Arc<RwLock<HashMap<String, BoxExporter>>>,
    // serviceName: registryUrls
    services: HashMap<String, Vec<Url>>,
}

impl Debug for RegistryProtocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            format!(
                "RegistryProtocol services{:#?},registries,{:#?}",
                self.services,
                self.registries.clone()
            )
            .as_str(),
        )
    }
}

impl RegistryProtocol {
    pub fn new() -> Self {
        RegistryProtocol {
            registries: None,
            exporters: Arc::new(RwLock::new(HashMap::new())),
            services: HashMap::new(),
        }
    }

    pub fn with_registries(mut self, registries: Registries) -> Self {
        self.registries = Some(registries);
        self
    }

    pub fn with_services(mut self, services: HashMap<String, Vec<Url>>) -> Self {
        self.services.extend(services);
        self
    }

    pub fn get_registry(&mut self, url: Url) -> BoxRegistry {
        let mem = MemoryRegistry::default();
        self.registries
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .insert(url.location, Arc::new(Mutex::new(Box::new(mem.clone()))));

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
        let registry_url = self.services.get(url.get_service_name().as_str());
        if let Some(urls) = registry_url {
            for url in urls.clone().iter() {
                if !url.service_key.is_empty() {
                    let mut reg = self.get_registry(url.clone());
                    reg.register(url.clone()).unwrap();
                }
            }
        }

        match url.clone().scheme.as_str() {
            "tri" => {
                let pro = Box::new(TripleProtocol::new());
                return pro.export(url).await;
            }
            _ => {
                tracing::error!("base {:?} not implemented", url.scheme);
                Box::new(TripleExporter::new())
            }
        }
    }

    async fn refer(self, url: Url) -> Self::Invoker {
        // getRegisterUrl
        // get Registry from registry_url
        // init directory based on registry_url and Registry
        // init Cluster based on Directory generates Invoker
        todo!()
        //Box::new(TripleInvoker::new(url))
    }
}

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
use std::error::Error;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use futures::future;
use futures::Future;
use tracing::{debug, info};

use dubbo_config::protocol::ProtocolRetrieve;
use dubbo_config::{get_global_config, RootConfig};

use crate::common::url::Url;
use crate::protocol::{BoxExporter, Protocol};
use crate::registry::protocol::RegistryProtocol;
use crate::registry::types::{Registries, RegistriesOperation};
use crate::registry::{BoxRegistry, Registry};

// Invoker是否可以基于hyper写一个通用的

#[derive(Default)]
pub struct Dubbo {
    // cached protocols, key of map means protocol name eg. dubbo, triple, grpc
    protocols: HashMap<String, Vec<Url>>,
    registries: Option<Registries>,
    service_registry: HashMap<String, Vec<Url>>, // registry: Urls
    config: Option<RootConfig>,
}

impl Dubbo {
    pub fn new() -> Dubbo {
        tracing_subscriber::fmt::init();
        Self {
            protocols: HashMap::new(),
            registries: None,
            service_registry: HashMap::new(),
            config: None,
        }
    }

    pub fn with_config(mut self, c: RootConfig) -> Self {
        self.config = Some(c);
        self
    }

    pub fn add_registry(mut self, registry_key: &str, registry: BoxRegistry) -> Self {
        if self.registries.is_none() {
            self.registries = Some(Arc::new(Mutex::new(HashMap::new())));
        }
        self.registries
            .as_ref()
            .unwrap()
            .insert(registry_key.to_string(), Arc::new(Mutex::new(registry)));
        self
    }

    pub fn init(&mut self) -> Result<(), Box<dyn Error>> {
        if self.config.is_none() {
            self.config = Some(get_global_config())
        }

        let root_config = self.config.as_ref().unwrap();
        debug!("global conf: {:?}", root_config);
        // env::set_var("ZOOKEEPER_SERVERS",root_config);
        for (_, service_config) in root_config.provider.services.iter() {
            info!("init service name: {}", service_config.interface);
            let url = if root_config
                .protocols
                .contains_key(service_config.protocol.as_str())
            {
                let protocol = root_config
                    .protocols
                    .get_protocol_or_default(service_config.protocol.as_str());
                let protocol_url =
                    format!("{}/{}", protocol.to_url(), service_config.interface.clone(),);
                info!("protocol_url: {:?}", protocol_url);
                Url::from_url(&protocol_url)
            } else {
                return Err(format!("protocol {:?} not exists", service_config.protocol).into());
            };
            info!("url: {:?}", url);
            if url.is_none() {
                continue;
            }
            let u = url.unwrap();
            if self.protocols.get(&service_config.protocol).is_some() {
                self.protocols
                    .get_mut(&service_config.protocol)
                    .unwrap()
                    .push(u);
            } else {
                self.protocols
                    .insert(service_config.protocol.clone(), vec![u]);
            }
        }
        Ok(())
    }

    pub async fn start(&mut self) {
        self.init().unwrap();
        info!("starting...");
        // TODO: server registry
        let mem_reg = Box::new(
            RegistryProtocol::new()
                .with_registries(self.registries.as_ref().unwrap().clone())
                .with_services(self.service_registry.clone()),
        );
        let mut async_vec: Vec<Pin<Box<dyn Future<Output = BoxExporter> + Send>>> = Vec::new();
        for (name, items) in self.protocols.iter() {
            for url in items.iter() {
                info!("protocol: {:?}, service url: {:?}", name, url);
                let exporter = mem_reg.clone().export(url.to_owned());
                async_vec.push(exporter);
                //TODO multiple registry
                if self.registries.is_some() {
                    self.registries
                        .as_ref()
                        .unwrap()
                        .default_registry()
                        .register(url.clone())
                        .unwrap();
                }
            }
        }

        let _res = future::join_all(async_vec).await;
    }
}

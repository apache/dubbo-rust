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

use std::cell::RefCell;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use futures::future;
use futures::Future;
use tracing::info;

use dubbo_config::{get_global_config, RootConfig};

use crate::common::url::Url;
use crate::protocol::{BoxExporter, Protocol};
use crate::registry::protocol::RegistryProtocol;
use crate::registry::BoxRegistry;

// Invoker是否可以基于hyper写一个通用的

#[derive(Default)]
pub struct Dubbo {
    protocols: HashMap<String, Vec<Url>>,
    #[deprecated]
    registries: HashMap<String, Url>,
    #[deprecated]
    service_registry: HashMap<String, Vec<Url>>, // registry: Urls
    config: Option<RootConfig>,
    use_external_registry: bool,
    external_registry: Option<Arc<Mutex<BoxRegistry>>>,
}

impl Dubbo {
    pub fn new() -> Dubbo {
        tracing_subscriber::fmt::init();
        Self {
            protocols: HashMap::new(),
            registries: HashMap::new(),
            service_registry: HashMap::new(),
            config: None,
            use_external_registry: false,
            external_registry: None,
        }
    }

    pub fn with_config(mut self, c: RootConfig) -> Self {
        self.config = Some(c);
        self
    }

    pub fn with_external_registry(mut self, registry: BoxRegistry) -> Self {
        self.use_external_registry = true;
        self.external_registry = Some(Arc::new(Mutex::new(registry)));
        self
    }

    pub fn init(&mut self) {
        if self.config.is_none() {
            self.config = Some(get_global_config())
        }

        let conf = self.config.as_ref().unwrap();
        tracing::debug!("global conf: {:?}", conf);

        for (_, c) in conf.provider.services.iter() {
            let u = if c.protocol_configs.is_empty() {
                let protocol = match conf.protocols.get(&c.protocol) {
                    Some(v) => v.to_owned(),
                    None => {
                        tracing::warn!("protocol {:?} not exists", c.protocol);
                        continue;
                    }
                };
                let protocol_url = format!("{}/{}", protocol.to_url(), c.name.clone(),);
                Url::from_url(&protocol_url)
            } else {
                let protocol = match c.protocol_configs.get(&c.protocol) {
                    Some(v) => v.to_owned(),
                    None => {
                        tracing::warn!("protocol {:?} not exists", c.protocol);
                        continue;
                    }
                };
                let protocol_url = format!("{}/{}", protocol.to_url(), c.name.clone(),);
                Url::from_url(&protocol_url)
            };
            info!("url: {:?}", u);
            if u.is_none() {
                continue;
            }

            let u = u.unwrap();

            if self.protocols.get(&c.protocol).is_some() {
                self.protocols.get_mut(&c.protocol).unwrap().push(u);
            } else {
                self.protocols.insert(c.protocol.clone(), vec![u]);
            }
        }
    }

    pub async fn start(&mut self) {
        self.init();
        info!("starting...");
        // TODO: server registry
        let mem_reg =
            Box::new(RegistryProtocol::new().with_services(self.service_registry.clone()));
        info!("{:?}", mem_reg);
        let mut async_vec: Vec<Pin<Box<dyn Future<Output = BoxExporter> + Send>>> = Vec::new();
        for (name, items) in self.protocols.iter() {
            for url in items.iter() {
                info!("protocol: {:?}, service url: {:?}", name, url);
                let exporter = mem_reg.clone().export(url.to_owned());
                async_vec.push(exporter);
                if self.use_external_registry {
                    let external_registry = self.external_registry.clone().unwrap();
                    external_registry
                        .lock()
                        .unwrap()
                        .register(url.clone())
                        .expect("registry err.");
                }
            }
        }

        let _res = future::join_all(async_vec).await;
    }
}

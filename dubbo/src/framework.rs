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

use std::{collections::HashMap, error::Error, pin::Pin};

use crate::{
    config::{get_global_config, protocol::ProtocolRetrieve, RootConfig},
    extension,
    extension::registry_extension::Registry,
    logger::tracing::{debug, info},
    protocol::{BoxExporter, Protocol},
    registry::protocol::RegistryProtocol,
    triple::server::support::{RpcHttp2Server, RpcServer},
    Url,
};
use futures::{future, Future};

// Invoker是否可以基于hyper写一个通用的

#[derive(Default)]
pub struct Dubbo {
    protocols: HashMap<String, Vec<Url>>,
    registries: Vec<Url>,
    service_registry: HashMap<String, Vec<Url>>, // registry: Urls
    config: Option<&'static RootConfig>,
}

impl Dubbo {
    pub fn new() -> Dubbo {
        Self {
            protocols: HashMap::new(),
            registries: Vec::default(),
            service_registry: HashMap::new(),
            config: None,
        }
    }

    pub fn with_config(mut self, config: RootConfig) -> Self {
        self.config = Some(config.leak());
        self
    }

    pub fn add_registry(mut self, registry: &str) -> Self {
        let url: Url = registry.parse().unwrap();
        let url = extension::registry_extension::to_extension_url(url);
        self.registries.push(url);
        self
    }

    pub fn register_server<T: RpcServer>(self, server: T) -> Self {
        let info = server.get_info();
        let server_name = info.0.to_owned();
        let s: RpcHttp2Server<T> = RpcHttp2Server::new(server);
        crate::protocol::triple::TRIPLE_SERVICES
            .write()
            .unwrap()
            .insert(
                server_name,
                crate::utils::boxed_clone::BoxCloneService::new(s),
            );
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
                let interface_name = service_config.interface.clone();
                let mut protocol_url = format!(
                    "{}/{}?interface={}&category={}&protocol={}",
                    protocol.to_url(),
                    interface_name,
                    interface_name,
                    "providers",
                    "tri"
                );
                if let Some(serialization) = &service_config.serialization {
                    protocol_url.push_str(&format!("&prefer.serialization={}", serialization));
                }
                info!("protocol_url: {:?}", protocol_url);
                protocol_url.parse().ok()
            } else {
                return Err(format!("base {:?} not exists", service_config.protocol).into());
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

        let mut registry_extensions = Vec::new();

        for registry_url in &self.registries {
            let registry_url = registry_url.clone();
            let registry_extension = extension::EXTENSIONS.load_registry(registry_url).await;
            if let Ok(registry_extension) = registry_extension {
                registry_extensions.push(registry_extension);
            }
        }

        let mem_reg = Box::new(
            RegistryProtocol::new()
                .with_registries(registry_extensions.clone())
                .with_services(self.service_registry.clone()),
        );
        let mut async_vec: Vec<Pin<Box<dyn Future<Output = BoxExporter> + Send>>> = Vec::new();
        for (name, items) in self.protocols.iter() {
            for url in items.iter() {
                info!("base: {:?}, service url: {:?}", name, url);
                let exporter = mem_reg.clone().export(url.to_owned());
                async_vec.push(exporter);

                //TODO multiple registry
                for registry_extension in &registry_extensions {
                    let _ = registry_extension.register(url.clone()).await;
                }
            }
        }

        let _res = future::join_all(async_vec).await;
    }
}

impl Drop for Dubbo {
    fn drop(&mut self) {
        unsafe {
            if let Some(config) = self.config.take() {
                let _ = Box::from_raw(config as *const RootConfig as *mut RootConfig);
            }
        }
    }
}

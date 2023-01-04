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
use std::pin::Pin;

use futures::future;
use futures::Future;

use crate::common::url::Url;
use crate::protocol::{BoxExporter, Protocol};
use crate::registry::protocol::RegistryProtocol;
use dubbo_config::{get_global_config, RootConfig};

// Invoker是否可以基于hyper写一个通用的

#[derive(Default)]
pub struct Dubbo {
    protocols: HashMap<String, Vec<Url>>,
    registries: HashMap<String, Url>,
    service_registry: HashMap<String, Vec<Url>>, // registry: Urls
    config: Option<&'static RootConfig>,
}

impl Dubbo {
    pub fn new() -> Dubbo {
        tracing_subscriber::fmt::init();
        Self {
            protocols: HashMap::new(),
            registries: HashMap::new(),
            service_registry: HashMap::new(),
            config: None,
        }
    }

    pub fn with_config(mut self, config: RootConfig) -> Self {
        self.config = Some(config.leak());
        self
    }

    pub fn init(&mut self) {
        let conf = self.config.get_or_insert_with(get_global_config);
        tracing::debug!("global conf: {:?}", conf);

        for (name, url) in conf.registries.iter() {
            self.registries.insert(
                name.clone(),
                Url::from_url(url).expect(&format!("url: {url} parse failed.")),
            );
        }

        for (service_name, service_config) in conf.provider.services.iter() {
            let protocol_url = match service_config
                .protocol_configs
                .get(&service_config.protocol)
            {
                Some(protocol_url) => protocol_url,
                None => {
                    let Some(protocol_url) = conf.protocols.get(&service_config.protocol) else {
                                    tracing::warn!("protocol: {:?} not exists", service_config.protocol);
                                    continue;
                                };
                    protocol_url
                }
            };
            // let protocol_url = format!(
            //     "{}/{}/{}",
            //     &protocol_url.to_url(),
            //     service_config.name,
            //     service_name
            // );
            // service_names may be multiple
            let protocol_url = protocol_url
                .to_owned()
                .add_param("service_names".to_string(), service_name.to_string());
            let protocol_url = protocol_url.to_url();
            tracing::info!("url: {}", protocol_url);

            let protocol_url = Url::from_url(&protocol_url)
                .expect(&format!("protocol url: {protocol_url} parse failed."));
            self.protocols
                .entry(service_config.name.clone())
                .and_modify(|urls| urls.push(protocol_url.clone()))
                .or_insert(vec![protocol_url]);

            tracing::debug!(
                "service name: {service_name}, service_config: {:?}",
                service_config
            );
            let registry = &service_config.registry;
            let reg_url = self
                .registries
                .get(registry)
                .expect(&format!("can't find the registry: {registry}"));
            self.service_registry
                .entry(service_config.name.clone())
                .and_modify(|urls| urls.push(reg_url.to_owned()))
                .or_insert(vec![reg_url.to_owned()]);
        }
    }

    pub async fn start(&mut self) {
        self.init();

        // TODO: server registry

        let mem_reg =
            Box::new(RegistryProtocol::new().with_services(self.service_registry.clone()));
        let mut async_vec: Vec<Pin<Box<dyn Future<Output = BoxExporter> + Send>>> = Vec::new();
        for (name, items) in self.protocols.iter() {
            for url in items.iter() {
                tracing::info!("protocol: {:?}, service url: {:?}", name, url);
                let exporter = mem_reg.clone().export(url.to_owned());
                async_vec.push(exporter)
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

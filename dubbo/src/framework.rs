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
use futures::FutureExt;

use crate::common::url::Url;
use crate::protocol::triple::triple_invoker::TripleInvoker;
use crate::protocol::triple::triple_protocol::TripleProtocol;
use crate::protocol::{Exporter, Protocol};
use config::{get_global_config, RootConfig};

pub type BoxExporter = Box<dyn Exporter<InvokerType = TripleInvoker>>;
// Invoker是否可以基于hyper写一个通用的

#[derive(Default)]
pub struct Dubbo {
    protocols: HashMap<String, Vec<Url>>,
    config: Option<RootConfig>,
}

impl Dubbo {
    pub fn new() -> Dubbo {
        Self {
            protocols: HashMap::new(),
            config: None,
        }
    }

    pub fn with_config(mut self, c: RootConfig) -> Self {
        self.config = Some(c);
        self
    }

    pub fn init(&mut self) {
        tracing_subscriber::fmt::init();

        if self.config.is_none() {
            self.config = Some(get_global_config())
        }

        let conf = self.config.as_ref().unwrap();
        tracing::debug!("global conf: {:?}", conf);
        for (_, c) in conf.service.iter() {
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
            tracing::info!("url: {:?}", u);
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

        // TODO: server registry

        let mut async_vec: Vec<Pin<Box<dyn Future<Output = BoxExporter> + Send>>> = Vec::new();
        for (key, c) in self.protocols.iter() {
            match key.as_str() {
                "triple" => {
                    let pro = Box::new(TripleProtocol::new());
                    for u in c.iter() {
                        let tri_fut = pro
                            .clone()
                            .export(u.clone())
                            .map(|res| Box::new(res) as BoxExporter)
                            .boxed();
                        async_vec.push(tri_fut);
                    }
                }
                _ => {
                    tracing::error!("protocol {:?} not implemented", key);
                }
            }
        }

        let _res = future::join_all(async_vec).await;
    }
}

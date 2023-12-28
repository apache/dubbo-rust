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

use std::sync::Arc;

use crate::{
    cluster::NewCluster,
    directory::NewCachedDirectory,
    loadbalancer::NewLoadBalancer,
    registry::n_registry::{ArcRegistry, RegistryComponent, StaticRegistry},
    route::NewRoutes,
    utils::boxed_clone::BoxCloneService,
};

use aws_smithy_http::body::SdkBody;
use dubbo_base::Url;
use tower::ServiceBuilder;

pub type ClientBoxService =
    BoxCloneService<http::Request<SdkBody>, http::Response<crate::BoxBody>, crate::Error>;

pub type ServiceMK = Arc<NewCluster<NewLoadBalancer<NewRoutes<NewCachedDirectory<ArcRegistry>>>>>;

#[derive(Default)]
pub struct ClientBuilder {
    pub timeout: Option<u64>,
    pub connector: &'static str,
    registry: Option<ArcRegistry>,
    pub direct: bool,
    host: String,
}

impl ClientBuilder {
    pub fn new() -> ClientBuilder {
        ClientBuilder {
            timeout: None,
            connector: "",
            registry: None,
            direct: false,
            host: "".to_string(),
        }
    }

    pub fn from_static(host: &str) -> ClientBuilder {
        Self {
            timeout: None,
            connector: "",
            registry: Some(ArcRegistry::new(StaticRegistry::new(vec![Url::from_url(
                host,
            )
            .unwrap()]))),
            direct: true,
            host: host.to_string(),
        }
    }

    pub fn with_timeout(self, timeout: u64) -> Self {
        Self {
            timeout: Some(timeout),
            ..self
        }
    }

    pub fn with_registry(self, registry: ArcRegistry) -> Self {
        Self {
            registry: Some(registry),
            ..self
        }
    }

    pub fn with_host(self, host: &'static str) -> Self {
        Self {
            registry: Some(ArcRegistry::new(StaticRegistry::new(vec![Url::from_url(
                host,
            )
            .unwrap()]))),
            ..self
        }
    }

    pub fn with_connector(self, connector: &'static str) -> Self {
        Self { connector, ..self }
    }

    pub fn with_direct(self, direct: bool) -> Self {
        Self { direct, ..self }
    }

    pub fn build(mut self) -> ServiceMK {
        let registry = self.registry.take().expect("registry must not be empty");

        let mk_service = ServiceBuilder::new()
            .layer(NewCluster::layer())
            .layer(NewLoadBalancer::layer())
            .layer(NewRoutes::layer())
            .layer(NewCachedDirectory::layer())
            .service(registry);

        Arc::new(mk_service)
    }
}

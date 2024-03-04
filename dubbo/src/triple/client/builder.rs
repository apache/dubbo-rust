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
    cluster::NewCluster, directory::NewCachedDirectory, extension, loadbalancer::NewLoadBalancer,
    route::NewRoutes, utils::boxed_clone::BoxCloneService,
};

use crate::registry::n_registry::StaticRegistryExtensionLoader;
use crate::registry::MkRegistryService;
use aws_smithy_http::body::SdkBody;
use dubbo_base::Url;
use tower::ServiceBuilder;

pub type ClientBoxService =
    BoxCloneService<http::Request<SdkBody>, http::Response<crate::BoxBody>, crate::Error>;

pub type ServiceMK =
    Arc<NewCluster<NewLoadBalancer<NewRoutes<NewCachedDirectory<MkRegistryService>>>>>;

#[derive(Default)]
pub struct ClientBuilder {
    pub timeout: Option<u64>,
    pub connector: &'static str,
    registry_extension_url: Option<Url>,
    pub direct: bool,
    host: String,
}

impl ClientBuilder {
    pub fn new() -> ClientBuilder {
        ClientBuilder {
            timeout: None,
            connector: "",
            registry_extension_url: None,
            direct: false,
            host: "".to_string(),
        }
    }

    pub fn from_static(host: &str) -> ClientBuilder {
        let registry_extension_url =
            StaticRegistryExtensionLoader::to_extension_url(vec![host.parse().unwrap()]);
        Self {
            timeout: None,
            connector: "",
            registry_extension_url: Some(registry_extension_url),
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

    pub fn with_registry(self, registry: Url) -> Self {
        let registry_extension_url = extension::registry_extension::to_extension_url(registry);
        Self {
            registry_extension_url: Some(registry_extension_url),
            ..self
        }
    }

    pub fn with_host(self, host: &'static str) -> Self {
        let registry_extension_url =
            StaticRegistryExtensionLoader::to_extension_url(vec![host.parse().unwrap()]);

        Self {
            registry_extension_url: Some(registry_extension_url),
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
        let registry = self
            .registry_extension_url
            .take()
            .expect("registry must not be empty");

        let mk_service = ServiceBuilder::new()
            .layer(NewCluster::layer())
            .layer(NewLoadBalancer::layer())
            .layer(NewRoutes::layer())
            .layer(NewCachedDirectory::layer())
            .service(MkRegistryService::new(registry));

        Arc::new(mk_service)
    }
}

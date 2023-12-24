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
mod utils;

use async_trait::async_trait;
use dubbo_base::Url;
use std::{collections::HashMap, sync::Arc};
use tokio::{select, sync::mpsc};

use anyhow::anyhow;
use dubbo::{
    registry::n_registry::{DiscoverStream, Registry, ServiceChange},
    StdError,
};
use dubbo_logger::tracing::{debug, error, info};
use nacos_sdk::api::naming::{
    NamingEventListener, NamingService, NamingServiceBuilder, ServiceInstance,
};

use crate::utils::{build_nacos_client_props, is_concrete_str, is_wildcard_str, match_range};

const VERSION_KEY: &str = "version";

const GROUP_KEY: &str = "group";

const DEFAULT_GROUP: &str = "DEFAULT_GROUP";

const PROVIDER_SIDE: &str = "provider";

const DEFAULT_CATEGORY: &str = PROVIDERS_CATEGORY;

const SIDE_KEY: &str = "side";

const REGISTER_CONSUMER_URL_KEY: &str = "register-consumer-url";

const SERVICE_NAME_SEPARATOR: &str = ":";

const CATEGORY_KEY: &str = "category";

const PROVIDERS_CATEGORY: &str = "providers";

#[allow(dead_code)]
const ADMIN_PROTOCOL: &str = "admin";

#[allow(dead_code)]
const INNERCLASS_SYMBOL: &str = "$";

#[allow(dead_code)]
const INNERCLASS_COMPATIBLE_SYMBOL: &str = "___";

pub struct NacosRegistry {
    nacos_naming_service: Arc<dyn NamingService + Sync + Send + 'static>,
}

impl NacosRegistry {
    pub fn new(url: Url) -> Self {
        let (nacos_client_props, enable_auth) = build_nacos_client_props(&url);

        let mut nacos_naming_builder = NamingServiceBuilder::new(nacos_client_props);

        if enable_auth {
            nacos_naming_builder = nacos_naming_builder.enable_auth_plugin_http();
        }

        let nacos_naming_service = nacos_naming_builder.build().unwrap();

        Self {
            nacos_naming_service: Arc::new(nacos_naming_service),
        }
    }
}

impl NacosRegistry {
    fn create_nacos_service_instance(url: Url) -> ServiceInstance {
        let ip = url.ip;
        let port = url.port;
        nacos_sdk::api::naming::ServiceInstance {
            ip,
            port: port.parse().unwrap(),
            metadata: url.params,
            ..Default::default()
        }
    }

    fn diff<'a>(
        old_service: &'a Vec<ServiceInstance>,
        new_services: &'a Vec<ServiceInstance>,
    ) -> (Vec<&'a ServiceInstance>, Vec<&'a ServiceInstance>) {
        let new_hosts_map: HashMap<String, &ServiceInstance> = new_services
            .iter()
            .map(|hosts| (hosts.ip_and_port(), hosts))
            .collect();

        let old_hosts_map: HashMap<String, &ServiceInstance> = old_service
            .iter()
            .map(|hosts| (hosts.ip_and_port(), hosts))
            .collect();

        let mut add_hosts = Vec::<&ServiceInstance>::new();
        let mut removed_hosts = Vec::<&ServiceInstance>::new();

        for (key, new_host) in new_hosts_map.iter() {
            let old_host = old_hosts_map.get(key);
            match old_host {
                None => {
                    add_hosts.push(*new_host);
                }
                Some(old_host) => {
                    if !old_host.is_same_instance(new_host) {
                        removed_hosts.push(*old_host);
                        add_hosts.push(*new_host);
                    }
                }
            }
        }

        for (key, old_host) in old_hosts_map.iter() {
            let new_host = new_hosts_map.get(key);
            match new_host {
                None => {
                    removed_hosts.push(*old_host);
                }
                Some(_) => {}
            }
        }

        (removed_hosts, add_hosts)
    }
}

#[async_trait]
impl Registry for NacosRegistry {
    async fn register(&self, url: Url) -> Result<(), dubbo::StdError> {
        // let side = url.get_param(SIDE_KEY).unwrap_or_default();
        // let register_consumer = url
        //     .get_param(REGISTER_CONSUMER_URL_KEY)
        //     .unwrap_or_else(|| false.to_string())
        //     .parse::<bool>()
        //     .unwrap_or(false);
        // if side.ne(PROVIDER_SIDE) && !register_consumer {
        //     warn!("Please set 'dubbo.registry.parameters.register-consumer-url=true' to turn on consumer url registration.");
        //     return Ok(());
        // }

        let nacos_service_name = NacosServiceName::new(&url);

        let group_name = Some(
            nacos_service_name
                .get_group_with_default(DEFAULT_GROUP)
                .to_string(),
        );
        let nacos_service_name = nacos_service_name.to_register_str();

        let nacos_service_instance = Self::create_nacos_service_instance(url);

        info!("register service: {}", nacos_service_name);
        let ret = self
            .nacos_naming_service
            .register_instance(nacos_service_name, group_name, nacos_service_instance)
            .await;
        if let Err(e) = ret {
            error!("register to nacos occur an error: {:?}", e);
            return Err(anyhow!("register to nacos occur an error: {:?}", e).into());
        }

        Ok(())
    }

    async fn unregister(&self, url: Url) -> Result<(), dubbo::StdError> {
        let nacos_service_name = NacosServiceName::new(&url);

        let group_name = Some(
            nacos_service_name
                .get_group_with_default(DEFAULT_GROUP)
                .to_string(),
        );
        let nacos_service_name = nacos_service_name.to_register_str();

        let nacos_service_instance = Self::create_nacos_service_instance(url);

        info!("deregister service: {}", nacos_service_name);

        let ret = self
            .nacos_naming_service
            .deregister_instance(nacos_service_name, group_name, nacos_service_instance)
            .await;
        if let Err(e) = ret {
            error!("deregister service from nacos occur an error: {:?}", e);
            return Err(anyhow!("deregister service from nacos occur an error: {:?}", e).into());
        }
        Ok(())
    }

    async fn subscribe(&self, url: Url) -> Result<DiscoverStream, StdError> {
        let service_name = NacosServiceName::new(&url);
        let service_group = service_name
            .get_group_with_default(DEFAULT_GROUP)
            .to_string();
        let subscriber_url = service_name.to_subscriber_str();
        info!("subscribe: {}", subscriber_url);

        let (listener, mut change_receiver) = ServiceChangeListener::new();
        let arc_listener = Arc::new(listener);

        let (discover_tx, discover_rx) = mpsc::channel(64);

        let nacos_naming_service = self.nacos_naming_service.clone();

        let listener_in_task = arc_listener.clone();
        let service_group_in_task = service_group.clone();
        let subscriber_url_in_task = subscriber_url.clone();
        tokio::spawn(async move {
            let listener = listener_in_task;
            let service_group = service_group_in_task;
            let subscriber_url = subscriber_url_in_task;

            let mut current_instances = Vec::new();
            loop {
                let change = select! {
                    _ = discover_tx.closed() => {
                        debug!("service {} change task quit, unsubscribe.", subscriber_url);
                        None
                    },
                    change = change_receiver.recv() => change
                };

                match change {
                    Some(instances) => {
                        debug!("service {} changed", subscriber_url);
                        let (remove_instances, add_instances) =
                            NacosRegistry::diff(&current_instances, &instances);

                        for instance in remove_instances {
                            let service_name = instance.service_name.as_ref();
                            let url = match service_name {
                                None => {
                                    format!("triple://{}:{}", instance.ip(), instance.port())
                                }
                                Some(service_name) => {
                                    format!(
                                        "triple://{}:{}/{}",
                                        instance.ip(),
                                        instance.port(),
                                        service_name
                                    )
                                }
                            };

                            match discover_tx.send(Ok(ServiceChange::Remove(url))).await {
                                Ok(_) => {}
                                Err(e) => {
                                    error!(
                                        "send service change failed: {:?}, maybe user unsubscribe",
                                        e
                                    );
                                    break;
                                }
                            }
                        }

                        for instance in add_instances {
                            let service_name = instance.service_name.as_ref();
                            let url = match service_name {
                                None => {
                                    format!("triple://{}:{}", instance.ip(), instance.port())
                                }
                                Some(service_name) => {
                                    format!(
                                        "triple://{}:{}/{}",
                                        instance.ip(),
                                        instance.port(),
                                        service_name
                                    )
                                }
                            };

                            match discover_tx.send(Ok(ServiceChange::Insert(url, ()))).await {
                                Ok(_) => {}
                                Err(e) => {
                                    error!(
                                        "send service change failed: {:?}, maybe user unsubscribe",
                                        e
                                    );
                                    break;
                                }
                            }
                        }
                        current_instances = instances;
                    }
                    None => {
                        error!(
                            "receive service change task quit, unsubscribe {}.",
                            subscriber_url
                        );
                        break;
                    }
                }
            }

            debug!("unsubscribe service: {}", subscriber_url);
            // unsubscribe
            let unsubscribe = nacos_naming_service
                .unsubscribe(subscriber_url, Some(service_group), Vec::new(), listener)
                .await;

            match unsubscribe {
                Ok(_) => {}
                Err(e) => {
                    error!("unsubscribe service failed: {:?}", e);
                }
            }
        });

        let all_instance = self
            .nacos_naming_service
            .get_all_instances(
                subscriber_url.clone(),
                Some(service_group.clone()),
                Vec::new(),
                false,
            )
            .await?;
        let _ = arc_listener.changed(all_instance);

        match self
            .nacos_naming_service
            .subscribe(
                subscriber_url.clone(),
                Some(service_group.clone()),
                Vec::new(),
                arc_listener,
            )
            .await
        {
            Ok(_) => {}
            Err(e) => {
                error!("subscribe service failed: {:?}", e);
                return Err(anyhow!("subscribe service failed: {:?}", e).into());
            }
        }

        Ok(discover_rx)
    }

    async fn unsubscribe(&self, url: Url) -> Result<(), dubbo::StdError> {
        let service_name = NacosServiceName::new(&url);
        let subscriber_url = service_name.to_subscriber_str();
        info!("unsubscribe: {}", &subscriber_url);

        Ok(())
    }
}

struct NacosServiceName {
    category: String,

    service_interface: String,

    version: String,

    group: String,
}

impl NacosServiceName {
    fn new(url: &Url) -> NacosServiceName {
        let service_interface = url.get_service_name();

        let category = url.get_param(CATEGORY_KEY).unwrap_or_default();

        let version = url.get_param(VERSION_KEY).unwrap_or_default();

        let group = url.get_param(GROUP_KEY).unwrap_or_default();

        Self {
            category,
            service_interface: service_interface.clone(),
            version,
            group,
        }
    }

    #[allow(dead_code)]
    fn from_service_name_str(service_name_str: &str) -> Self {
        let mut splitter = service_name_str.split(SERVICE_NAME_SEPARATOR);

        let category = splitter.next().unwrap_or_default().to_string();
        let service_interface = splitter.next().unwrap_or_default().to_string();
        let version = splitter.next().unwrap_or_default().to_string();
        let group = splitter.next().unwrap_or_default().to_string();

        Self {
            category,
            service_interface,
            version,
            group,
        }
    }

    #[allow(dead_code)]
    fn version(&self) -> &str {
        &self.version
    }

    #[allow(dead_code)]
    fn get_version_with_default<'a>(&'a self, default: &'a str) -> &str {
        if self.version.is_empty() {
            default
        } else {
            &self.version
        }
    }

    #[allow(dead_code)]
    fn group(&self) -> &str {
        &self.group
    }

    fn get_group_with_default<'a>(&'a self, default: &'a str) -> &str {
        if self.group.is_empty() {
            default
        } else {
            &self.group
        }
    }

    #[allow(dead_code)]
    fn category(&self) -> &str {
        &self.category
    }

    #[allow(dead_code)]
    fn get_category_with_default<'a>(&'a self, default: &'a str) -> &str {
        if self.category.is_empty() {
            default
        } else {
            &self.category
        }
    }

    #[allow(dead_code)]
    fn service_interface(&self) -> &str {
        &self.service_interface
    }

    #[allow(dead_code)]
    fn get_service_interface_with_default<'a>(&'a self, default: &'a str) -> &str {
        if self.service_interface.is_empty() {
            default
        } else {
            &self.service_interface
        }
    }

    fn to_register_str(&self) -> String {
        let category = if self.category.is_empty() {
            DEFAULT_CATEGORY
        } else {
            &self.category
        };
        format!(
            "{}:{}:{}:{}",
            category, self.service_interface, self.version, self.group
        )
    }

    fn to_subscriber_str(&self) -> String {
        let category = if is_concrete_str(&self.service_interface) {
            DEFAULT_CATEGORY
        } else {
            &self.category
        };

        format!(
            "{}:{}:{}:{}",
            category, self.service_interface, self.version, self.group
        )
    }

    #[allow(dead_code)]
    fn to_subscriber_legacy_string(&self) -> String {
        let mut legacy_string = DEFAULT_CATEGORY.to_owned();
        if !self.service_interface.is_empty() {
            legacy_string.push_str(SERVICE_NAME_SEPARATOR);
            legacy_string.push_str(&self.service_interface);
        }

        if !self.version.is_empty() {
            legacy_string.push_str(SERVICE_NAME_SEPARATOR);
            legacy_string.push_str(&self.version);
        }

        if !self.group.is_empty() {
            legacy_string.push_str(SERVICE_NAME_SEPARATOR);
            legacy_string.push_str(&self.group);
        }

        legacy_string
    }

    #[allow(dead_code)]
    fn is_concrete(&self) -> bool {
        is_concrete_str(&self.service_interface)
            && is_concrete_str(&self.version)
            && is_concrete_str(&self.group)
    }

    #[allow(dead_code)]
    fn is_compatible(&self, other: &NacosServiceName) -> bool {
        if !other.is_concrete() {
            return false;
        }

        if !self.category.eq(&other.category) && !match_range(&self.category, &other.category) {
            return false;
        }

        if is_wildcard_str(&self.version) {
            return true;
        }

        if is_wildcard_str(&self.group) {
            return true;
        }

        if !&self.version.eq(&other.version) && !match_range(&self.version, &other.version) {
            return false;
        }

        if !self.group.eq(&other.group) && !match_range(&self.group, &other.group) {
            return false;
        }

        true
    }
}

struct ServiceChangeListener {
    tx: mpsc::Sender<Vec<ServiceInstance>>,
}

impl ServiceChangeListener {
    pub fn new() -> (Self, mpsc::Receiver<Vec<ServiceInstance>>) {
        let (tx, rx) = mpsc::channel(64);
        let this = Self { tx };

        (this, rx)
    }

    pub fn changed(&self, instances: Vec<ServiceInstance>) -> Result<(), dubbo::StdError> {
        match self.tx.try_send(instances) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("send service change failed: {:?}", e);
                Err(anyhow!("send service change failed: {:?}", e).into())
            }
        }
    }
}

impl NamingEventListener for ServiceChangeListener {
    fn event(&self, event: Arc<nacos_sdk::api::naming::NamingChangeEvent>) {
        debug!("service change {}", event.service_name.clone());
        debug!("nacos event: {:?}", event);

        let instances = event.instances.as_ref();
        match instances {
            None => {
                let _ = self.changed(Vec::default());
            }
            Some(instances) => {
                let _ = self.changed(instances.clone());
            }
        }
    }
}

#[cfg(test)]
pub mod tests {

    use core::time;
    use std::thread;

    use tracing::metadata::LevelFilter;

    use super::*;

    #[tokio::test]
    #[ignore]
    pub async fn test_register_to_nacos() {
        tracing_subscriber::fmt()
            .with_thread_names(true)
            .with_file(true)
            .with_level(true)
            .with_line_number(true)
            .with_thread_ids(true)
            .with_max_level(LevelFilter::DEBUG)
            .init();

        let nacos_registry_url = Url::from_url("nacos://127.0.0.1:8848/org.apache.dubbo.registry.RegistryService?application=dubbo-demo-triple-api-provider&dubbo=2.0.2&interface=org.apache.dubbo.registry.RegistryService&pid=7015").unwrap();
        let registry = NacosRegistry::new(nacos_registry_url);

        let mut service_url = Url::from_url("tri://127.0.0.1:50052/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&methods=sayHello,sayHelloAsync&pid=7015&service-name-mapping=true&side=provider&timestamp=1670060843807").unwrap();
        service_url
            .params
            .insert(SIDE_KEY.to_owned(), PROVIDER_SIDE.to_owned());

        let ret = registry.register(service_url).await;

        info!("register result: {:?}", ret);

        let sleep_millis = time::Duration::from_secs(300);
        thread::sleep(sleep_millis);
    }

    #[tokio::test]
    #[ignore]
    pub async fn test_register_and_unregister() {
        tracing_subscriber::fmt()
            .with_thread_names(true)
            .with_file(true)
            .with_level(true)
            .with_line_number(true)
            .with_thread_ids(true)
            .with_max_level(LevelFilter::DEBUG)
            .init();

        let nacos_registry_url = Url::from_url("nacos://127.0.0.1:8848/org.apache.dubbo.registry.RegistryService?application=dubbo-demo-triple-api-provider&dubbo=2.0.2&interface=org.apache.dubbo.registry.RegistryService&pid=7015").unwrap();
        let registry = NacosRegistry::new(nacos_registry_url);

        let mut service_url = Url::from_url("tri://127.0.0.1:9090/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&methods=sayHello,sayHelloAsync&pid=7015&service-name-mapping=true&side=provider&timestamp=1670060843807").unwrap();
        service_url
            .params
            .insert(SIDE_KEY.to_owned(), PROVIDER_SIDE.to_owned());

        let ret = registry.register(service_url).await;

        info!("register result: {:?}", ret);

        let sleep_millis = time::Duration::from_secs(10);
        thread::sleep(sleep_millis);

        let unregister_url = Url::from_url("tri://127.0.0.1:9090/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&methods=sayHello,sayHelloAsync&pid=7015&service-name-mapping=true&side=provider&timestamp=1670060843807").unwrap();
        let ret = registry.unregister(unregister_url).await;

        info!("deregister result: {:?}", ret);

        let sleep_millis = time::Duration::from_secs(10);
        thread::sleep(sleep_millis);
    }

    #[tokio::test]
    #[ignore]
    pub async fn test_subscribe() {
        tracing_subscriber::fmt()
            .with_thread_names(true)
            .with_file(true)
            .with_level(true)
            .with_line_number(true)
            .with_thread_ids(true)
            .with_max_level(LevelFilter::DEBUG)
            .init();

        let nacos_registry_url = Url::from_url("nacos://127.0.0.1:8848/org.apache.dubbo.registry.RegistryService?application=dubbo-demo-triple-api-provider&dubbo=2.0.2&interface=org.apache.dubbo.registry.RegistryService&pid=7015").unwrap();
        let registry = NacosRegistry::new(nacos_registry_url);

        let mut service_url = Url::from_url("tri://127.0.0.1:50052/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&methods=sayHello,sayHelloAsync&pid=7015&service-name-mapping=true&side=provider&timestamp=1670060843807").unwrap();
        service_url
            .params
            .insert(SIDE_KEY.to_owned(), PROVIDER_SIDE.to_owned());

        let ret = registry.register(service_url).await;

        info!("register result: {:?}", ret);

        let subscribe_url = Url::from_url("consumer://192.168.0.102:50052/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&bind.ip=192.168.0.102&bind.port=50052&category=configurators&check=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&ipv6=fd00:6cb1:58a2:8ddf:0:0:0:1000&methods=sayHello,sayHelloAsync&pid=44270&service-name-mapping=true&side=provider").unwrap();
        let subscribe_ret = registry.subscribe(subscribe_url).await;

        if let Err(e) = subscribe_ret {
            error!("error message: {:?}", e);
            return;
        }

        let mut rx = subscribe_ret.unwrap();
        let change = rx.recv().await;
        info!("receive change: {:?}", change);

        let sleep_millis = time::Duration::from_secs(300);
        thread::sleep(sleep_millis);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore]
    pub async fn test_unsubscribe() {
        tracing_subscriber::fmt()
            .with_thread_names(true)
            .with_file(true)
            .with_level(true)
            .with_line_number(true)
            .with_thread_ids(true)
            .with_max_level(LevelFilter::DEBUG)
            .init();

        let nacos_registry_url = Url::from_url("nacos://127.0.0.1:8848/org.apache.dubbo.registry.RegistryService?application=dubbo-demo-triple-api-provider&dubbo=2.0.2&interface=org.apache.dubbo.registry.RegistryService&pid=7015").unwrap();
        let registry = NacosRegistry::new(nacos_registry_url);

        let mut service_url = Url::from_url("tri://127.0.0.1:50052/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&methods=sayHello,sayHelloAsync&pid=7015&service-name-mapping=true&side=provider&timestamp=1670060843807").unwrap();
        service_url
            .params
            .insert(SIDE_KEY.to_owned(), PROVIDER_SIDE.to_owned());

        let ret = registry.register(service_url).await;

        info!("register result: {:?}", ret);

        let subscribe_url = Url::from_url("provider://192.168.0.102:50052/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&bind.ip=192.168.0.102&bind.port=50052&category=configurators&check=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&ipv6=fd00:6cb1:58a2:8ddf:0:0:0:1000&methods=sayHello,sayHelloAsync&pid=44270&service-name-mapping=true&side=provider").unwrap();

        let ret = registry.subscribe(subscribe_url).await;

        if let Err(e) = ret {
            error!("error message: {:?}", e);
            return;
        }

        let mut rx = ret.unwrap();
        let change = rx.recv().await;
        info!("receive change: {:?}", change);

        let sleep_millis = time::Duration::from_secs(40);
        thread::sleep(sleep_millis);

        let unsubscribe_url = Url::from_url("provider://192.168.0.102:50052/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&bind.ip=192.168.0.102&bind.port=50052&category=configurators&check=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&ipv6=fd00:6cb1:58a2:8ddf:0:0:0:1000&methods=sayHello,sayHelloAsync&pid=44270&service-name-mapping=true&side=provider").unwrap();
        let ret = registry.unsubscribe(unsubscribe_url).await;

        if let Err(e) = ret {
            error!("error message: {:?}", e);
            return;
        }

        let sleep_millis = time::Duration::from_secs(40);
        thread::sleep(sleep_millis);
    }
}

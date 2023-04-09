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

use dubbo_base::Url;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use anyhow::anyhow;
use dubbo::registry::{NotifyListener, Registry, RegistryNotifyListener, ServiceEvent};
use dubbo_logger::tracing::{error, info, warn};
use nacos_sdk::api::naming::{NamingService, NamingServiceBuilder, ServiceInstance};

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
    listeners: Mutex<HashMap<String, HashSet<Arc<NotifyListenerWrapper>>>>,
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
            listeners: Mutex::new(HashMap::new()),
        }
    }

    #[allow(dead_code)]
    fn get_subscribe_service_names(&self, service_name: &NacosServiceName) -> HashSet<String> {
        if service_name.is_concrete() {
            let mut set = HashSet::new();
            let service_subscribe_name = service_name.to_subscriber_str();
            let service_subscriber_legacy_name = service_name.to_subscriber_legacy_string();
            if service_subscribe_name.eq(&service_subscriber_legacy_name) {
                set.insert(service_subscribe_name);
            } else {
                set.insert(service_subscribe_name);
                set.insert(service_subscriber_legacy_name);
            }

            set
        } else {
            let list_view = self.nacos_naming_service.get_service_list(
                1,
                i32::MAX,
                Some(
                    service_name
                        .get_group_with_default(DEFAULT_GROUP)
                        .to_string(),
                ),
            );
            if let Err(e) = list_view {
                error!("list service instances occur an error: {:?}", e);
                return HashSet::default();
            }

            let list_view = list_view.unwrap();
            let set: HashSet<String> = list_view
                .0
                .into_iter()
                .filter(|service_name| service_name.split(SERVICE_NAME_SEPARATOR).count() == 4)
                .map(|service_name| NacosServiceName::from_service_name_str(&service_name))
                .filter(|other_service_name| service_name.is_compatible(other_service_name))
                .map(|service_name| service_name.to_subscriber_str())
                .collect();
            set
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
}

impl Registry for NacosRegistry {
    fn register(&mut self, url: Url) -> Result<(), dubbo::StdError> {
        let side = url.get_param(SIDE_KEY).unwrap_or_default();
        let register_consumer = url
            .get_param(REGISTER_CONSUMER_URL_KEY)
            .unwrap_or_else(|| false.to_string())
            .parse::<bool>()
            .unwrap_or(false);
        if side.ne(PROVIDER_SIDE) && !register_consumer {
            warn!("Please set 'dubbo.registry.parameters.register-consumer-url=true' to turn on consumer url registration.");
            return Ok(());
        }

        let nacos_service_name = NacosServiceName::new(&url);

        let group_name = Some(
            nacos_service_name
                .get_group_with_default(DEFAULT_GROUP)
                .to_string(),
        );
        let nacos_service_name = nacos_service_name.to_register_str();

        let nacos_service_instance = Self::create_nacos_service_instance(url);

        info!("register service: {}", nacos_service_name);
        let ret = self.nacos_naming_service.register_instance(
            nacos_service_name,
            group_name,
            nacos_service_instance,
        );
        if let Err(e) = ret {
            error!("register to nacos occur an error: {:?}", e);
            return Err(anyhow!("register to nacos occur an error: {:?}", e).into());
        }

        Ok(())
    }

    fn unregister(&mut self, url: Url) -> Result<(), dubbo::StdError> {
        let nacos_service_name = NacosServiceName::new(&url);

        let group_name = Some(
            nacos_service_name
                .get_group_with_default(DEFAULT_GROUP)
                .to_string(),
        );
        let nacos_service_name = nacos_service_name.to_register_str();

        let nacos_service_instance = Self::create_nacos_service_instance(url);

        info!("deregister service: {}", nacos_service_name);

        let ret = self.nacos_naming_service.deregister_instance(
            nacos_service_name,
            group_name,
            nacos_service_instance,
        );
        if let Err(e) = ret {
            error!("deregister service from nacos occur an error: {:?}", e);
            return Err(anyhow!("deregister service from nacos occur an error: {:?}", e).into());
        }
        Ok(())
    }

    fn subscribe(&self, url: Url, listener: RegistryNotifyListener) -> Result<(), dubbo::StdError> {
        let service_name = NacosServiceName::new(&url);
        let url_str = url.to_url();

        info!("subscribe: {}", &url_str);

        let nacos_listener: Arc<NotifyListenerWrapper> = {
            let listeners = self.listeners.lock();
            if let Err(e) = listeners {
                error!("subscribe service failed: {:?}", e);
                return Err(anyhow!("subscribe service failed: {:?}", e).into());
            }

            let mut listeners = listeners.unwrap();
            let listener_set = listeners.get_mut(url_str.as_str());

            let wrapper = Arc::new(NotifyListenerWrapper(listener));
            if let Some(listener_set) = listener_set {
                listener_set.insert(wrapper.clone());
            } else {
                let mut hash_set = HashSet::new();
                hash_set.insert(wrapper.clone());
                listeners.insert(url_str, hash_set);
            }

            wrapper
        };

        let ret = self.nacos_naming_service.subscribe(
            service_name.to_subscriber_str(),
            Some(
                service_name
                    .get_group_with_default(DEFAULT_GROUP)
                    .to_string(),
            ),
            Vec::new(),
            nacos_listener,
        );

        if let Err(e) = ret {
            error!("subscribe service failed: {:?}", e);
            return Err(anyhow!("subscribe service failed: {:?}", e).into());
        }

        Ok(())
    }

    fn unsubscribe(
        &self,
        url: Url,
        listener: RegistryNotifyListener,
    ) -> Result<(), dubbo::StdError> {
        let service_name = NacosServiceName::new(&url);
        let url_str = url.to_url();
        info!("unsubscribe: {}", &url_str);

        let nacos_listener: Arc<NotifyListenerWrapper> = {
            let listeners = self.listeners.lock();
            if let Err(e) = listeners {
                error!("unsubscribe service failed: {:?}", e);
                return Err(anyhow!("unsubscribe service failed: {:?}", e).into());
            }

            let mut listeners = listeners.unwrap();
            let listener_set = listeners.get_mut(url_str.as_str());
            if listener_set.is_none() {
                return Ok(());
            }

            let listener_set = listener_set.unwrap();

            let listener = Arc::new(NotifyListenerWrapper(listener));
            let listener = listener_set.take(&listener);
            if listener.is_none() {
                return Ok(());
            }

            listener.unwrap()
        };

        let ret = self.nacos_naming_service.unsubscribe(
            service_name.to_subscriber_str(),
            Some(
                service_name
                    .get_group_with_default(DEFAULT_GROUP)
                    .to_string(),
            ),
            Vec::new(),
            nacos_listener,
        );

        if let Err(e) = ret {
            error!("unsubscribe service failed: {:?}", e);
            return Err(anyhow!("unsubscribe service failed: {:?}", e).into());
        }

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

struct NotifyListenerWrapper(Arc<dyn NotifyListener + Sync + Send + 'static>);

impl std::hash::Hash for NotifyListenerWrapper {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let ptr = self.0.as_ref();
        std::ptr::hash(ptr, state);
    }
}

impl PartialEq for NotifyListenerWrapper {
    fn eq(&self, other: &Self) -> bool {
        let self_ptr = self.0.as_ref() as *const dyn NotifyListener;
        let other_ptr = other.0.as_ref() as *const dyn NotifyListener;

        let (self_data_ptr, _): (*const u8, *const u8) = unsafe { std::mem::transmute(self_ptr) };

        let (other_data_ptr, _): (*const u8, *const u8) = unsafe { std::mem::transmute(other_ptr) };
        self_data_ptr == other_data_ptr
    }
}

impl Eq for NotifyListenerWrapper {}

impl nacos_sdk::api::naming::NamingEventListener for NotifyListenerWrapper {
    fn event(&self, event: Arc<nacos_sdk::api::naming::NamingChangeEvent>) {
        let service_name = event.service_name.clone();
        let instances = event.instances.as_ref();
        let urls: Vec<Url>;
        if let Some(instances) = instances {
            urls = instances
                .iter()
                .filter_map(|data| {
                    let url_str =
                        format!("triple://{}:{}/{}", data.ip(), data.port(), service_name);
                    Url::from_url(&url_str)
                })
                .collect();
        } else {
            urls = Vec::new();
        }
        let notify_event = ServiceEvent {
            key: service_name,
            action: String::from("CHANGE"),
            service: urls,
        };
        self.0.notify(notify_event);
    }
}

#[cfg(test)]
pub mod tests {

    use core::time;
    use std::thread;

    use tracing::metadata::LevelFilter;

    use super::*;

    #[test]
    #[ignore]
    pub fn test_register_to_nacos() {
        tracing_subscriber::fmt()
            .with_thread_names(true)
            .with_file(true)
            .with_level(true)
            .with_line_number(true)
            .with_thread_ids(true)
            .with_max_level(LevelFilter::DEBUG)
            .init();

        let nacos_registry_url = Url::from_url("nacos://127.0.0.1:8848/org.apache.dubbo.registry.RegistryService?application=dubbo-demo-triple-api-provider&dubbo=2.0.2&interface=org.apache.dubbo.registry.RegistryService&pid=7015").unwrap();
        let mut registry = NacosRegistry::new(nacos_registry_url);

        let mut service_url = Url::from_url("tri://127.0.0.1:50052/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&methods=sayHello,sayHelloAsync&pid=7015&service-name-mapping=true&side=provider&timestamp=1670060843807").unwrap();
        service_url
            .params
            .insert(SIDE_KEY.to_owned(), PROVIDER_SIDE.to_owned());

        let ret = registry.register(service_url);

        info!("register result: {:?}", ret);

        let sleep_millis = time::Duration::from_secs(300);
        thread::sleep(sleep_millis);
    }

    #[test]
    #[ignore]
    pub fn test_register_and_unregister() {
        tracing_subscriber::fmt()
            .with_thread_names(true)
            .with_file(true)
            .with_level(true)
            .with_line_number(true)
            .with_thread_ids(true)
            .with_max_level(LevelFilter::DEBUG)
            .init();

        let nacos_registry_url = Url::from_url("nacos://127.0.0.1:8848/org.apache.dubbo.registry.RegistryService?application=dubbo-demo-triple-api-provider&dubbo=2.0.2&interface=org.apache.dubbo.registry.RegistryService&pid=7015").unwrap();
        let mut registry = NacosRegistry::new(nacos_registry_url);

        let mut service_url = Url::from_url("tri://127.0.0.1:9090/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&methods=sayHello,sayHelloAsync&pid=7015&service-name-mapping=true&side=provider&timestamp=1670060843807").unwrap();
        service_url
            .params
            .insert(SIDE_KEY.to_owned(), PROVIDER_SIDE.to_owned());

        let ret = registry.register(service_url);

        info!("register result: {:?}", ret);

        let sleep_millis = time::Duration::from_secs(10);
        thread::sleep(sleep_millis);

        let unregister_url = Url::from_url("tri://127.0.0.1:9090/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&methods=sayHello,sayHelloAsync&pid=7015&service-name-mapping=true&side=provider&timestamp=1670060843807").unwrap();
        let ret = registry.unregister(unregister_url);

        info!("deregister result: {:?}", ret);

        let sleep_millis = time::Duration::from_secs(10);
        thread::sleep(sleep_millis);
    }

    struct TestNotifyListener;
    impl NotifyListener for TestNotifyListener {
        fn notify(&self, event: ServiceEvent) {
            info!("notified: {:?}", event.key);
        }

        fn notify_all(&self, event: ServiceEvent) {
            info!("notify_all: {:?}", event.key);
        }
    }

    #[test]
    #[ignore]
    fn test_subscribe() {
        tracing_subscriber::fmt()
            .with_thread_names(true)
            .with_file(true)
            .with_level(true)
            .with_line_number(true)
            .with_thread_ids(true)
            .with_max_level(LevelFilter::DEBUG)
            .init();

        let nacos_registry_url = Url::from_url("nacos://127.0.0.1:8848/org.apache.dubbo.registry.RegistryService?application=dubbo-demo-triple-api-provider&dubbo=2.0.2&interface=org.apache.dubbo.registry.RegistryService&pid=7015").unwrap();
        let mut registry = NacosRegistry::new(nacos_registry_url);

        let mut service_url = Url::from_url("tri://127.0.0.1:50052/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&methods=sayHello,sayHelloAsync&pid=7015&service-name-mapping=true&side=provider&timestamp=1670060843807").unwrap();
        service_url
            .params
            .insert(SIDE_KEY.to_owned(), PROVIDER_SIDE.to_owned());

        let ret = registry.register(service_url);

        info!("register result: {:?}", ret);

        let subscribe_url = Url::from_url("provider://192.168.0.102:50052/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&bind.ip=192.168.0.102&bind.port=50052&category=configurators&check=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&ipv6=fd00:6cb1:58a2:8ddf:0:0:0:1000&methods=sayHello,sayHelloAsync&pid=44270&service-name-mapping=true&side=provider").unwrap();

        let ret = registry.subscribe(subscribe_url, Arc::new(TestNotifyListener));

        if let Err(e) = ret {
            error!("error message: {:?}", e);
            return;
        }

        let sleep_millis = time::Duration::from_secs(300);
        thread::sleep(sleep_millis);
    }

    #[test]
    #[ignore]
    fn test_unsubscribe() {
        tracing_subscriber::fmt()
            .with_thread_names(true)
            .with_file(true)
            .with_level(true)
            .with_line_number(true)
            .with_thread_ids(true)
            .with_max_level(LevelFilter::DEBUG)
            .init();

        let nacos_registry_url = Url::from_url("nacos://127.0.0.1:8848/org.apache.dubbo.registry.RegistryService?application=dubbo-demo-triple-api-provider&dubbo=2.0.2&interface=org.apache.dubbo.registry.RegistryService&pid=7015").unwrap();
        let mut registry = NacosRegistry::new(nacos_registry_url);

        let mut service_url = Url::from_url("tri://127.0.0.1:50052/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&methods=sayHello,sayHelloAsync&pid=7015&service-name-mapping=true&side=provider&timestamp=1670060843807").unwrap();
        service_url
            .params
            .insert(SIDE_KEY.to_owned(), PROVIDER_SIDE.to_owned());

        let ret = registry.register(service_url);

        info!("register result: {:?}", ret);

        let subscribe_url = Url::from_url("provider://192.168.0.102:50052/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&bind.ip=192.168.0.102&bind.port=50052&category=configurators&check=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&ipv6=fd00:6cb1:58a2:8ddf:0:0:0:1000&methods=sayHello,sayHelloAsync&pid=44270&service-name-mapping=true&side=provider").unwrap();

        let listener = Arc::new(TestNotifyListener);

        let ret = registry.subscribe(subscribe_url, listener.clone());

        if let Err(e) = ret {
            error!("error message: {:?}", e);
            return;
        }

        let sleep_millis = time::Duration::from_secs(40);
        thread::sleep(sleep_millis);

        let unsubscribe_url = Url::from_url("provider://192.168.0.102:50052/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&bind.ip=192.168.0.102&bind.port=50052&category=configurators&check=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&ipv6=fd00:6cb1:58a2:8ddf:0:0:0:1000&methods=sayHello,sayHelloAsync&pid=44270&service-name-mapping=true&side=provider").unwrap();
        let ret = registry.unsubscribe(unsubscribe_url, listener.clone());

        if let Err(e) = ret {
            error!("error message: {:?}", e);
            return;
        }

        let sleep_millis = time::Duration::from_secs(40);
        thread::sleep(sleep_millis);
    }
}

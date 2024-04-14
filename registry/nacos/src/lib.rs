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

use async_trait::async_trait;
use dubbo::{url::UrlParam, StdError, Url};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;

use dubbo::{
    extension::{
        registry_extension::{DiscoverStream, Registry, ServiceChange},
        Extension,
    },
    logger::tracing::info,
    params::registry_param::{
        AppName, Category, Group, InterfaceName, RegistryUrl, ServiceNamespace, Version,
    },
};
use nacos_sdk::api::{
    naming::{NamingEventListener, NamingService, NamingServiceBuilder, ServiceInstance},
    props::ClientProps,
};
use tokio::sync::{watch, Notify};

pub struct NacosRegistry {
    url: Url,
    nacos_service: Arc<dyn NamingService + Send + Sync>,
}

impl NacosRegistry {
    pub fn new(url: Url, nacos_service: Arc<dyn NamingService + Send + Sync>) -> Self {
        Self { url, nacos_service }
    }

    fn create_nacos_service_instance(url: &Url) -> ServiceInstance {
        let ip = url.host().unwrap();
        let port = url.port().unwrap();

        ServiceInstance {
            ip: ip.to_string(),
            port: port.into(),
            metadata: url.all_query_params(),
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
    async fn register(&self, url: Url) -> Result<(), StdError> {
        let service_name = NacosServiceName::new(&url);

        let group_name = service_name.group();

        let registry_service_name_str = service_name.value();

        let service_instance = Self::create_nacos_service_instance(&url);

        self.nacos_service
            .register_instance(
                registry_service_name_str.to_owned(),
                Some(group_name.to_owned()),
                service_instance,
            )
            .await?;

        Ok(())
    }

    async fn unregister(&self, url: Url) -> Result<(), StdError> {
        let service_name = NacosServiceName::new(&url);

        let group_name = service_name.group();

        let registry_service_name_str = service_name.value();

        let service_instance = Self::create_nacos_service_instance(&url);

        self.nacos_service
            .deregister_instance(
                registry_service_name_str.to_owned(),
                Some(group_name.to_owned()),
                service_instance,
            )
            .await?;

        Ok(())
    }

    async fn subscribe(&self, url: Url) -> Result<DiscoverStream, StdError> {
        let service_name = NacosServiceName::new(&url);

        let group_name = service_name.group().to_owned();

        let registry_service_name_str = service_name.value().to_owned();

        let all_instance = self
            .nacos_service
            .get_all_instances(
                registry_service_name_str.to_owned(),
                Some(group_name.to_owned()),
                Vec::default(),
                false,
            )
            .await?;

        let (tx, rx) = mpsc::channel(1024);

        let (event_listener, mut listener_change_rx, closed) = NacosNamingEventListener::new();
        let event_listener = Arc::new(event_listener);

        let nacos_service_cloned = self.nacos_service.clone();
        let event_listener_cloned = event_listener.clone();
        let registry_service_name_str_clone = registry_service_name_str.clone();
        let group_name_clone = group_name.clone();
        tokio::spawn(async move {
            let mut current_instances = all_instance;
            for instance in &current_instances {
                let url = instance_to_url(instance).as_str().to_owned();
                let _ = tx.send(Ok(ServiceChange::Insert(url, ()))).await;
            }

            loop {
                let change = tokio::select! {
                    _ = closed.notified() => {
                        break;
                    },
                    change = listener_change_rx.changed() => change
                };

                if change.is_err() {
                    break;
                }

                let change = listener_change_rx.borrow_and_update().clone();
                if change.is_empty() {
                    continue;
                }
                let (remove_instances, add_instances) = Self::diff(&current_instances, &change);

                for remove_instance in remove_instances {
                    let url = instance_to_url(remove_instance).as_str().to_owned();
                    let Ok(_) = tx.send(Ok(ServiceChange::Remove(url))).await else {
                        break;
                    };
                }

                for add_instance in add_instances {
                    let url = instance_to_url(add_instance).as_str().to_owned();
                    let Ok(_) = tx.send(Ok(ServiceChange::Insert(url, ()))).await else {
                        break;
                    };
                }

                current_instances = change;
            }

            info!("unsubscribe");
            let _ = nacos_service_cloned
                .unsubscribe(
                    registry_service_name_str_clone,
                    Some(group_name_clone),
                    Vec::default(),
                    event_listener_cloned,
                )
                .await;
        });

        let _ = self
            .nacos_service
            .subscribe(
                registry_service_name_str,
                Some(group_name),
                Vec::default(),
                event_listener,
            )
            .await?;

        Ok(rx)
    }

    async fn unsubscribe(&self, _: Url) -> Result<(), StdError> {
        Ok(())
    }

    fn url(&self) -> &Url {
        &self.url
    }
}

#[async_trait]
impl Extension for NacosRegistry {
    type Target = Box<dyn Registry + Send + Sync + 'static>;

    fn name() -> String {
        "nacos".to_string()
    }

    async fn create(url: Url) -> Result<Self::Target, StdError> {
        // url example:
        // extension://0.0.0.0?extension-type=registry&extension-name=nacos&registry=nacos://127.0.0.1:8848
        let registry_url = url.query::<RegistryUrl>().unwrap();
        let registry_url = registry_url.value();

        let host = registry_url.host().unwrap();
        let port = registry_url.port().unwrap_or(8848);

        let nacos_server_addr = format!("{}:{}", host, port);

        let namespace = registry_url.query::<ServiceNamespace>().unwrap_or_default();
        let namespace = namespace.value();

        let app_name = registry_url.query::<AppName>().unwrap_or_default();
        let app_name = app_name.value();

        let user_name = registry_url.username();
        let password = registry_url.password().unwrap_or_default();

        let nacos_client_props = ClientProps::new()
            .server_addr(nacos_server_addr)
            .namespace(namespace)
            .app_name(app_name)
            .auth_username(user_name)
            .auth_password(password);

        let mut nacos_naming_builder = NamingServiceBuilder::new(nacos_client_props);

        if !user_name.is_empty() {
            nacos_naming_builder = nacos_naming_builder.enable_auth_plugin_http();
        }

        let nacos_naming_service = nacos_naming_builder.build().unwrap();

        let nacos_registry = NacosRegistry::new(registry_url, Arc::new(nacos_naming_service));

        Ok(Box::new(nacos_registry))
    }
}

fn instance_to_url(instance: &ServiceInstance) -> Url {
    let mut url = Url::empty();
    url.set_protocol("provider");
    url.set_host(instance.ip());
    url.set_port(instance.port().try_into().unwrap_or_default());
    url.extend_pairs(
        instance
            .metadata()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone())),
    );

    url
}

struct NacosNamingEventListener {
    tx: watch::Sender<Vec<ServiceInstance>>,
    closed: Arc<Notify>,
}

impl NacosNamingEventListener {
    fn new() -> (Self, watch::Receiver<Vec<ServiceInstance>>, Arc<Notify>) {
        let (tx, rx) = watch::channel(Vec::new());

        let closed = Arc::new(Notify::new());
        let this = Self {
            tx,
            closed: closed.clone(),
        };
        (this, rx, closed)
    }
}

impl NamingEventListener for NacosNamingEventListener {
    fn event(&self, event: Arc<nacos_sdk::api::naming::NamingChangeEvent>) {
        match event.instances {
            Some(ref instances) => {
                let instances = instances.clone();
                let send = self.tx.send(instances);
                match send {
                    Ok(_) => {}
                    Err(_) => {
                        self.closed.notify_waiters();
                    }
                }
            }
            None => {}
        }
    }
}

struct NacosServiceName {
    #[allow(dead_code)]
    category: String,

    #[allow(dead_code)]
    interface: String,

    #[allow(dead_code)]
    version: String,

    #[allow(dead_code)]
    group: String,

    #[allow(dead_code)]
    value: String,
}

impl NacosServiceName {
    fn new(url: &Url) -> Self {
        let interface = url.query::<InterfaceName>().unwrap();
        let interface = interface.value();

        let category = url.query::<Category>().unwrap_or_default();
        let category = category.value();

        let version = url.query::<Version>().unwrap_or_default();
        let version = version.value();

        let group = url.query::<Group>().unwrap_or_default();
        let group = group.value();

        let value = format!("{}:{}:{}:{}", category, interface, version, group);

        Self {
            category,
            interface,
            version,
            group,
            value,
        }
    }

    #[allow(dead_code)]
    fn category(&self) -> &str {
        &self.category
    }

    #[allow(dead_code)]
    fn interface(&self) -> &str {
        &self.interface
    }

    #[allow(dead_code)]
    fn version(&self) -> &str {
        &self.version
    }

    #[allow(dead_code)]
    fn group(&self) -> &str {
        &self.group
    }

    #[allow(dead_code)]
    fn value(&self) -> &str {
        &self.value
    }
}

#[cfg(test)]
pub mod tests {

    use core::time;
    use std::thread;
    use tracing::error;

    use dubbo::params::{extension_param::ExtensionName, registry_param::Side};
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

        let mut extension_url: Url = "extension://0.0.0.0?extension-type=registry"
            .parse()
            .unwrap();
        extension_url.add_query_param(ExtensionName::new("nacos".to_string()));
        extension_url.add_query_param(RegistryUrl::new("nacos://127.0.0.1:8848/org.apache.dubbo.registry.RegistryService?application=dubbo-demo-triple-api-provider&dubbo=2.0.2&interface=org.apache.dubbo.registry.RegistryService&pid=7015".parse().unwrap()));

        let registry = NacosRegistry::create(extension_url).await.unwrap();

        let mut service_url: Url = "tri://127.0.0.1:50052/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&methods=sayHello,sayHelloAsync&pid=7015&service-name-mapping=true&side=provider&timestamp=1670060843807".parse().unwrap();

        service_url.add_query_param(Side::Provider);

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

        let mut extension_url: Url = "extension://0.0.0.0?extension-type=registry"
            .parse()
            .unwrap();
        extension_url.add_query_param(ExtensionName::new("nacos".to_string()));
        extension_url.add_query_param(RegistryUrl::new("nacos://127.0.0.1:8848/org.apache.dubbo.registry.RegistryService?application=dubbo-demo-triple-api-provider&dubbo=2.0.2&interface=org.apache.dubbo.registry.RegistryService&pid=7015".parse().unwrap()));

        let registry = NacosRegistry::create(extension_url).await.unwrap();

        let mut service_url: Url = "tri://127.0.0.1:50052/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&methods=sayHello,sayHelloAsync&pid=7015&service-name-mapping=true&side=provider&timestamp=1670060843807".parse().unwrap();

        service_url.add_query_param(Side::Provider);

        let ret = registry.register(service_url).await;

        info!("register result: {:?}", ret);

        let sleep_millis = time::Duration::from_secs(10);
        thread::sleep(sleep_millis);

        let unregister_url = "tri://127.0.0.1:9090/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&methods=sayHello,sayHelloAsync&pid=7015&service-name-mapping=true&side=provider&timestamp=1670060843807".parse().unwrap();
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

        let mut extension_url: Url = "extension://0.0.0.0?extension-type=registry"
            .parse()
            .unwrap();
        extension_url.add_query_param(ExtensionName::new("nacos".to_string()));
        extension_url.add_query_param(RegistryUrl::new("nacos://127.0.0.1:8848/org.apache.dubbo.registry.RegistryService?application=dubbo-demo-triple-api-provider&dubbo=2.0.2&interface=org.apache.dubbo.registry.RegistryService&pid=7015".parse().unwrap()));

        let registry = NacosRegistry::create(extension_url).await.unwrap();

        let mut service_url: Url = "tri://127.0.0.1:50052/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&methods=sayHello,sayHelloAsync&pid=7015&service-name-mapping=true&side=provider&timestamp=1670060843807".parse().unwrap();

        service_url.add_query_param(Side::Provider);

        let ret = registry.register(service_url).await;

        info!("register result: {:?}", ret);

        let subscribe_url = "consumer://192.168.0.102:50052/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&bind.ip=192.168.0.102&bind.port=50052&category=configurators&check=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&ipv6=fd00:6cb1:58a2:8ddf:0:0:0:1000&methods=sayHello,sayHelloAsync&pid=44270&service-name-mapping=true&side=provider".parse().unwrap();
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

        let mut extension_url: Url = "extension://0.0.0.0?extension-type=registry"
            .parse()
            .unwrap();
        extension_url.add_query_param(ExtensionName::new("nacos".to_string()));
        extension_url.add_query_param(RegistryUrl::new("nacos://127.0.0.1:8848/org.apache.dubbo.registry.RegistryService?application=dubbo-demo-triple-api-provider&dubbo=2.0.2&interface=org.apache.dubbo.registry.RegistryService&pid=7015".parse().unwrap()));

        let registry = NacosRegistry::create(extension_url).await.unwrap();

        let mut service_url: Url = "tri://127.0.0.1:50052/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&methods=sayHello,sayHelloAsync&pid=7015&service-name-mapping=true&side=provider&timestamp=1670060843807".parse().unwrap();

        service_url.add_query_param(Side::Provider);

        let ret = registry.register(service_url).await;

        info!("register result: {:?}", ret);

        let subscribe_url = "provider://192.168.0.102:50052/org.apache.dubbo.demo.GreeterService?anyhost=true&application=dubbo-demo-triple-api-provider&background=false&bind.ip=192.168.0.102&bind.port=50052&category=configurators&check=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=org.apache.dubbo.demo.GreeterService&ipv6=fd00:6cb1:58a2:8ddf:0:0:0:1000&methods=sayHello,sayHelloAsync&pid=44270&service-name-mapping=true&side=provider".parse().unwrap();

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

        drop(rx);

        let sleep_millis = time::Duration::from_secs(40);
        thread::sleep(sleep_millis);
    }
}

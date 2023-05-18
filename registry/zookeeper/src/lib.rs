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

#![allow(unused_variables, dead_code, missing_docs)]

use std::{
    collections::{HashMap, HashSet},
    env,
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

use dubbo_base::{
    constants::{DUBBO_KEY, LOCALHOST_IP, PROVIDERS_KEY},
    Url,
};
use dubbo_logger::tracing::{debug, error, info};
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use zookeeper::{Acl, CreateMode, WatchedEvent, WatchedEventType, Watcher, ZooKeeper};

use dubbo::{
    registry::{
        memory_registry::MemoryRegistry, NotifyListener, Registry, RegistryNotifyListener,
        ServiceEvent,
    },
    StdError,
};

// 从url中获取服务注册的元数据
// rawURL = fmt.Sprintf("%s://%s%s?%s", c.Protocol, host, c.Path, s)
// dubboPath = fmt.Sprintf("/%s/%s/%s", r.URL.GetParam(constant.RegistryGroupKey, "dubbo"), r.service(c), common.DubboNodes[common.PROVIDER])

pub const REGISTRY_GROUP_KEY: &str = "registry.group";

struct LoggingWatcher;
impl Watcher for LoggingWatcher {
    fn handle(&self, e: WatchedEvent) {
        info!("{:?}", e)
    }
}

//#[derive(Debug)]
pub struct ZookeeperRegistry {
    root_path: String,
    zk_client: Arc<ZooKeeper>,
    listeners: RwLock<HashMap<String, RegistryNotifyListener>>,
    memory_registry: Arc<Mutex<MemoryRegistry>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ZkServiceInstance {
    name: String,
    address: String,
    port: i32,
}

impl ZkServiceInstance {
    pub fn get_service_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_host(&self) -> &str {
        self.address.as_str()
    }

    pub fn get_port(&self) -> i32 {
        self.port
    }
}

impl ZookeeperRegistry {
    pub fn new(connect_string: &str) -> ZookeeperRegistry {
        let zk_client =
            ZooKeeper::connect(connect_string, Duration::from_secs(15), LoggingWatcher).unwrap();
        info!("zk server connect string: {}", connect_string);
        ZookeeperRegistry {
            root_path: "/services".to_string(),
            zk_client: Arc::new(zk_client),
            listeners: RwLock::new(HashMap::new()),
            memory_registry: Arc::new(Mutex::new(MemoryRegistry::default())),
        }
    }

    fn create_listener(
        &self,
        path: String,
        service_name: String,
        listener: RegistryNotifyListener,
    ) -> ServiceInstancesChangedListener {
        let mut service_names = HashSet::new();
        service_names.insert(service_name.clone());
        ServiceInstancesChangedListener {
            zk_client: Arc::clone(&self.zk_client),
            path,
            service_name: service_name.clone(),
            listener,
        }
    }

    // metadata /dubbo/mapping designed for application discovery; deprecated for currently using interface discovery
    // #[deprecated]
    fn get_app_name(&self, service_name: String) -> String {
        let res = self
            .zk_client
            .get_data(&("/dubbo/mapping/".to_owned() + &service_name), false);

        let x = res.unwrap().0;
        let s = match std::str::from_utf8(&x) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };
        s.to_string()
    }

    pub fn get_client(&self) -> Arc<ZooKeeper> {
        self.zk_client.clone()
    }

    // If the parent node does not exist in the ZooKeeper, Err(ZkError::NoNode) will be returned.
    pub fn create_path(
        &self,
        path: &str,
        data: &str,
        create_mode: CreateMode,
    ) -> Result<(), StdError> {
        if self.exists_path(path) {
            self.zk_client
                .set_data(path, data.as_bytes().to_vec(), None)
                .unwrap_or_else(|_| panic!("set data to {} failed.", path));
            return Ok(());
        }
        let zk_result = self.zk_client.create(
            path,
            data.as_bytes().to_vec(),
            Acl::open_unsafe().clone(),
            create_mode,
        );
        match zk_result {
            Ok(_) => Ok(()),
            Err(err) => {
                error!("zk path {} parent not exists.", path);
                Err(Box::try_from(err).unwrap())
            }
        }
    }

    // For avoiding Err(ZkError::NoNode) when parent node is't exists
    pub fn create_path_with_parent_check(
        &self,
        path: &str,
        data: &str,
        create_mode: CreateMode,
    ) -> Result<(), StdError> {
        let nodes: Vec<&str> = path.split('/').collect();
        let mut current: String = String::new();
        let children = *nodes.last().unwrap();
        for node_key in nodes {
            if node_key.is_empty() {
                continue;
            };
            current.push('/');
            current.push_str(node_key);
            if !self.exists_path(current.as_str()) {
                let new_create_mode = match children == node_key {
                    true => create_mode,
                    false => CreateMode::Persistent,
                };
                let new_data = match children == node_key {
                    true => data,
                    false => "",
                };
                self.create_path(current.as_str(), new_data, new_create_mode)
                    .unwrap();
            }
        }
        Ok(())
    }

    pub fn delete_path(&self, path: &str) {
        if self.exists_path(path) {
            self.get_client().delete(path, None).unwrap()
        }
    }

    pub fn exists_path(&self, path: &str) -> bool {
        self.zk_client.exists(path, false).unwrap().is_some()
    }

    pub fn get_data(&self, path: &str, watch: bool) -> Option<String> {
        if self.exists_path(path) {
            let zk_result = self.zk_client.get_data(path, watch);
            if let Ok(..) = zk_result {
                Some(String::from_utf8(zk_result.unwrap().0).unwrap())
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Default for ZookeeperRegistry {
    fn default() -> ZookeeperRegistry {
        let zk_connect_string = match env::var("ZOOKEEPER_SERVERS") {
            Ok(val) => val,
            Err(_) => {
                let default_connect_string = "localhost:2181";
                info!(
                    "No ZOOKEEPER_SERVERS env value, using {} as default.",
                    default_connect_string
                );
                default_connect_string.to_string()
            }
        };
        println!(
            "using external registry with it's connect string {}",
            zk_connect_string.as_str()
        );
        ZookeeperRegistry::new(zk_connect_string.as_str())
    }
}

impl Registry for ZookeeperRegistry {
    fn register(&mut self, url: Url) -> Result<(), StdError> {
        debug!("register url: {}", url);
        let zk_path = format!(
            "/{}/{}/{}/{}",
            DUBBO_KEY,
            url.service_name,
            PROVIDERS_KEY,
            url.encoded_raw_url_string()
        );
        self.create_path_with_parent_check(zk_path.as_str(), LOCALHOST_IP, CreateMode::Ephemeral)?;
        Ok(())
    }

    fn unregister(&mut self, url: Url) -> Result<(), StdError> {
        let zk_path = format!(
            "/{}/{}/{}/{}",
            DUBBO_KEY,
            url.service_name,
            PROVIDERS_KEY,
            url.encoded_raw_url_string()
        );
        self.delete_path(zk_path.as_str());
        Ok(())
    }

    // for consumer to find the changes of providers
    fn subscribe(&self, url: Url, listener: RegistryNotifyListener) -> Result<(), StdError> {
        let service_name = url.get_service_name();
        let zk_path = format!("/{}/{}/{}", DUBBO_KEY, &service_name, PROVIDERS_KEY);
        if self
            .listeners
            .read()
            .unwrap()
            .get(service_name.as_str())
            .is_some()
        {
            return Ok(());
        }

        self.listeners
            .write()
            .unwrap()
            .insert(service_name.to_string(), listener.clone());

        let zk_listener =
            self.create_listener(zk_path.clone(), service_name.to_string(), listener.clone());

        let zk_changed_paths = self.zk_client.get_children_w(&zk_path, zk_listener);
        let result = match zk_changed_paths {
            Err(err) => {
                error!("zk subscribe error: {}", err);
                Vec::new()
            }
            Ok(urls) => urls
                .iter()
                .map(|node_key| {
                    let provider_url: Url = urlencoding::decode(node_key)
                        .unwrap()
                        .to_string()
                        .as_str()
                        .into();
                    provider_url
                })
                .collect(),
        };
        info!("notifying {}->{:?}", service_name, result);
        listener.notify(ServiceEvent {
            key: service_name,
            action: String::from("ADD"),
            service: result,
        });
        Ok(())
    }

    fn unsubscribe(&self, url: Url, listener: RegistryNotifyListener) -> Result<(), StdError> {
        todo!()
    }
}

pub struct ServiceInstancesChangedListener {
    zk_client: Arc<ZooKeeper>,
    path: String,
    service_name: String,
    listener: RegistryNotifyListener,
}

impl Watcher for ServiceInstancesChangedListener {
    fn handle(&self, event: WatchedEvent) {
        if let (WatchedEventType::NodeChildrenChanged, Some(path)) = (event.event_type, event.path)
        {
            let event_path = path.clone();
            let dirs = self
                .zk_client
                .get_children(&event_path, false)
                .expect("msg");
            let result: Vec<Url> = dirs
                .iter()
                .map(|node_key| {
                    let provider_url: Url = node_key.as_str().into();
                    provider_url
                })
                .collect();
            let res = self.zk_client.get_children_w(
                &path,
                ServiceInstancesChangedListener {
                    zk_client: Arc::clone(&self.zk_client),
                    path: path.clone(),
                    service_name: self.service_name.clone(),
                    listener: Arc::clone(&self.listener),
                },
            );

            info!("notify {}->{:?}", self.service_name, result);
            self.listener.notify(ServiceEvent {
                key: self.service_name.clone(),
                action: String::from("ADD"),
                service: result,
            });
        }
    }
}

impl NotifyListener for ServiceInstancesChangedListener {
    fn notify(&self, event: ServiceEvent) {
        self.listener.notify(event);
    }

    fn notify_all(&self, event: ServiceEvent) {
        self.listener.notify(event);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use zookeeper::{Acl, CreateMode, WatchedEvent, Watcher};

    use crate::ZookeeperRegistry;

    struct TestZkWatcher {
        pub watcher: Arc<Option<TestZkWatcher>>,
    }

    impl Watcher for TestZkWatcher {
        fn handle(&self, event: WatchedEvent) {
            println!("event: {:?}", event);
        }
    }

    #[test]
    fn zk_read_write_watcher() {
        // https://github.com/bonifaido/rust-zookeeper/blob/master/examples/zookeeper_example.rs
        // using ENV to set zookeeper server urls
        let zkr = ZookeeperRegistry::default();
        let zk_client = zkr.get_client();
        let watcher = TestZkWatcher {
            watcher: Arc::new(None),
        };
        if zk_client.exists("/test", true).is_err() {
            zk_client
                .create(
                    "/test",
                    vec![1, 3],
                    Acl::open_unsafe().clone(),
                    CreateMode::Ephemeral,
                )
                .unwrap();
        }
        let zk_res = zk_client.create(
            "/test",
            "hello".into(),
            Acl::open_unsafe().clone(),
            CreateMode::Ephemeral,
        );
        let result = zk_client.get_children_w("/test", watcher);
        assert!(result.is_ok());
        if zk_client.exists("/test/a", true).is_err() {
            zk_client.delete("/test/a", None).unwrap();
        }
        if zk_client.exists("/test/a", true).is_err() {
            zk_client.delete("/test/b", None).unwrap();
        }
        let zk_res = zk_client.create(
            "/test/a",
            "hello".into(),
            Acl::open_unsafe().clone(),
            CreateMode::Ephemeral,
        );
        let zk_res = zk_client.create(
            "/test/b",
            "world".into(),
            Acl::open_unsafe().clone(),
            CreateMode::Ephemeral,
        );
        let test_a_result = zk_client.get_data("/test", true);
        assert!(test_a_result.is_ok());
        let vec1 = test_a_result.unwrap().0;
        // data in /test should equals to "hello"
        assert_eq!(String::from_utf8(vec1).unwrap(), "hello");
        zk_client.close().unwrap()
    }

    #[test]
    fn create_path_with_parent_check() {
        let zkr = ZookeeperRegistry::default();
        let path = "/du1bbo/test11111";
        let data = "hello";
        // creating a child on a not exists parent, throw a NoNode error.
        // let result = zkr.create_path(path, data, CreateMode::Ephemeral);
        // assert!(result.is_err());
        let create_with_parent_check_result =
            zkr.create_path_with_parent_check(path, data, CreateMode::Ephemeral);
        assert!(create_with_parent_check_result.is_ok());
        assert_eq!(data, zkr.get_data(path, false).unwrap());
    }
}

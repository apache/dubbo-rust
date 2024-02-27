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

use std::{collections::HashMap, env, sync::Arc, time::Duration};

use async_trait::async_trait;
use dubbo_base::{
    constants::{DUBBO_KEY, LOCALHOST_IP, PROVIDERS_KEY},
    Url,
};
use dubbo_logger::tracing::{debug, error, info};
use serde::{Deserialize, Serialize};
use tokio::{select, sync::mpsc};
use zookeeper::{Acl, CreateMode, WatchedEvent, WatchedEventType, Watcher, ZooKeeper};

use dubbo::{
    registry::n_registry::{DiscoverStream, Registry, ServiceChange},
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

pub struct ZookeeperRegistry {
    root_path: String,
    zk_client: Arc<ZooKeeper>,
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
                Err(err.into())
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
                let (new_create_mode, new_data) = match children == node_key {
                    true => (create_mode, data),
                    false => (CreateMode::Persistent, ""),
                };

                self.create_path(current.as_str(), new_data, new_create_mode)?;
            }
        }
        Ok(())
    }

    pub fn delete_path(&self, path: &str) {
        if self.exists_path(path) {
            self.zk_client.delete(path, None).unwrap()
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

    pub fn diff<'a>(
        old_urls: &'a Vec<String>,
        new_urls: &'a Vec<String>,
    ) -> (Vec<String>, Vec<String>) {
        let old_urls_map: HashMap<String, String> = old_urls
            .iter()
            .map(|url| dubbo_base::Url::from_url(url.as_str()))
            .filter(|item| item.is_some())
            .map(|item| item.unwrap())
            .map(|item| {
                let ip_port = item.get_ip_port();
                let url = item.encoded_raw_url_string();
                (ip_port, url)
            })
            .collect();

        let new_urls_map: HashMap<String, String> = new_urls
            .iter()
            .map(|url| dubbo_base::Url::from_url(url.as_str()))
            .filter(|item| item.is_some())
            .map(|item| item.unwrap())
            .map(|item| {
                let ip_port = item.get_ip_port();
                let url = item.encoded_raw_url_string();
                (ip_port, url)
            })
            .collect();

        let mut add_hosts = Vec::new();
        let mut removed_hosts = Vec::new();

        for (key, new_host) in new_urls_map.iter() {
            let old_host = old_urls_map.get(key);
            match old_host {
                None => {
                    add_hosts.push(new_host.clone());
                }
                Some(old_host) => {
                    if !old_host.eq(new_host) {
                        removed_hosts.push(old_host.clone());
                        add_hosts.push(new_host.clone());
                    }
                }
            }
        }

        for (key, old_host) in old_urls_map.iter() {
            let new_host = old_urls_map.get(key);
            match new_host {
                None => {
                    removed_hosts.push(old_host.clone());
                }
                Some(_) => {}
            }
        }

        (removed_hosts, add_hosts)
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

#[async_trait]
impl Registry for ZookeeperRegistry {
    async fn register(&self, url: Url) -> Result<(), StdError> {
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

    async fn unregister(&self, url: Url) -> Result<(), StdError> {
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
    async fn subscribe(&self, url: Url) -> Result<DiscoverStream, StdError> {
        let service_name = url.get_service_name();
        let zk_path = format!("/{}/{}/{}", DUBBO_KEY, &service_name, PROVIDERS_KEY);

        debug!("subscribe service: {}", zk_path);

        let (listener, mut change_rx) = ZooKeeperListener::new();
        let arc_listener = Arc::new(listener);

        let watcher = ZooKeeperWatcher::new(arc_listener.clone(), zk_path.clone());

        let (discover_tx, discover_rx) = mpsc::channel(64);

        let zk_client_in_task = self.zk_client.clone();
        let zk_path_in_task = zk_path.clone();
        let service_name_in_task = service_name.clone();
        let arc_listener_in_task = arc_listener.clone();
        tokio::spawn(async move {
            let zk_client = zk_client_in_task;
            let zk_path = zk_path_in_task;
            let service_name = service_name_in_task;
            let listener = arc_listener_in_task;

            let mut current_urls = Vec::new();

            loop {
                let changed = select! {
                    _ = discover_tx.closed() => {
                        info!("discover task quit, discover channel closed");
                        None
                    },
                    changed = change_rx.recv() => {
                        changed
                    }
                };

                match changed {
                    Some(_) => {
                        let zookeeper_watcher =
                            ZooKeeperWatcher::new(listener.clone(), zk_path.clone());

                        match zk_client.get_children_w(&zk_path, zookeeper_watcher) {
                            Ok(children) => {
                                let (removed, add) =
                                    ZookeeperRegistry::diff(&current_urls, &children);

                                for url in removed {
                                    match discover_tx
                                        .send(Ok(ServiceChange::Remove(url.clone())))
                                        .await
                                    {
                                        Ok(_) => {}
                                        Err(e) => {
                                            error!("send service change failed: {:?}, maybe user unsubscribe", e);
                                            break;
                                        }
                                    }
                                }

                                for url in add {
                                    match discover_tx
                                        .send(Ok(ServiceChange::Insert(url.clone(), ())))
                                        .await
                                    {
                                        Ok(_) => {}
                                        Err(e) => {
                                            error!("send service change failed: {:?}, maybe user unsubscribe", e);
                                            break;
                                        }
                                    }
                                }

                                current_urls = children;
                            }
                            Err(err) => {
                                error!("zk subscribe error: {}", err);
                                break;
                            }
                        }
                    }
                    None => {
                        error!("receive service change task quit, unsubscribe {}.", zk_path);
                        break;
                    }
                }
            }

            debug!("unsubscribe service: {}", zk_path);
        });

        arc_listener.changed(zk_path);

        Ok(discover_rx)
    }

    async fn unsubscribe(&self, url: Url) -> Result<(), StdError> {
        let service_name = url.get_service_name();
        let zk_path = format!("/{}/{}/{}", DUBBO_KEY, &service_name, PROVIDERS_KEY);

        info!("unsubscribe service: {}", zk_path);
        Ok(())
    }
}

pub struct ZooKeeperListener {
    tx: mpsc::Sender<String>,
}

impl ZooKeeperListener {
    pub fn new() -> (ZooKeeperListener, mpsc::Receiver<String>) {
        let (tx, rx) = mpsc::channel(64);
        let this = ZooKeeperListener { tx };
        (this, rx)
    }

    pub fn changed(&self, path: String) {
        match self.tx.try_send(path) {
            Ok(_) => {}
            Err(err) => {
                error!("send change list to listener occur an error: {}", err);
                return;
            }
        }
    }
}

pub struct ZooKeeperWatcher {
    listener: Arc<ZooKeeperListener>,
    path: String,
}

impl ZooKeeperWatcher {
    pub fn new(listener: Arc<ZooKeeperListener>, path: String) -> ZooKeeperWatcher {
        ZooKeeperWatcher { listener, path }
    }
}

impl Watcher for ZooKeeperWatcher {
    fn handle(&self, event: WatchedEvent) {
        info!("receive zookeeper event: {:?}", event);
        let event_type: WatchedEventType = event.event_type;
        match event_type {
            WatchedEventType::None => {
                info!("event type is none, ignore it.");
                return;
            }
            _ => {}
        }

        self.listener.changed(self.path.clone());
    }
}

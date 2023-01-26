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

use dubbo::common::url::Url;
use dubbo::registry::memory_registry::MemoryNotifyListener;
use dubbo::registry::NotifyListener;
use dubbo::registry::Registry;
use dubbo::registry::ServiceEvent;
use dubbo::StdError;
use serde::{Deserialize, Serialize};
use tracing::info;
use zookeeper::{Acl, CreateMode};
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;
use zookeeper::{WatchedEvent, WatchedEventType, Watcher, ZooKeeper};
use dubbo::cluster::support::cluster_invoker::ClusterInvoker;
use dubbo::codegen::BoxRegistry;
use dubbo::registry::integration::ClusterRegistryIntegration;

// 从url中获取服务注册的元数据
/// rawURL = fmt.Sprintf("%s://%s%s?%s", c.Protocol, host, c.Path, s)
/// dubboPath = fmt.Sprintf("/%s/%s/%s", r.URL.GetParam(constant.RegistryGroupKey, "dubbo"), r.service(c), common.DubboNodes[common.PROVIDER])

pub const REGISTRY_GROUP_KEY: &str = "registry.group";

struct LoggingWatcher;
impl Watcher for LoggingWatcher {
    fn handle(&self, e: WatchedEvent) {
        println!("{:?}", e)
    }
}

//#[derive(Debug)]
pub struct ZookeeperRegistry {
    root_path: String,
    zk_client: Arc<ZooKeeper>,

    listeners: RwLock<HashMap<String, Arc<<ZookeeperRegistry as Registry>::NotifyListener>>>,
}

pub struct MyNotifyListener {}

impl NotifyListener for MyNotifyListener {
    fn notify(&self, event: dubbo::registry::ServiceEvent) {
        todo!()
    }

    fn notify_all(&self, event: dubbo::registry::ServiceEvent) {
        todo!()
    }
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
        ZookeeperRegistry {
            root_path: "/services".to_string(),
            zk_client: Arc::new(zk_client),

            listeners: RwLock::new(HashMap::new()),
        }
    }

    fn create_listener(
        &self,
        path: String,
        service_name: String,
        listener: Arc<<ZookeeperRegistry as Registry>::NotifyListener>,
    ) -> ServiceInstancesChangedListener {
        let mut service_names = HashSet::new();
        service_names.insert(service_name.clone());
        return ServiceInstancesChangedListener {
            zk_client: Arc::clone(&self.zk_client),
            path: path,

            service_name: service_name.clone(),
            listener: listener,
        };
    }

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
}

impl Registry for ZookeeperRegistry {
    type NotifyListener = MemoryNotifyListener;

    fn register(&mut self, url: Url) -> Result<(), StdError> {
        todo!();
    }

    fn unregister(&mut self, url: Url) -> Result<(), StdError> {
        todo!();
    }

    fn subscribe(&self, url: Url, listener: Self::NotifyListener) -> Result<(), StdError> {
        let binding = url.get_service_name();
        let service_name = binding.get(0).unwrap();
        let app_name = self.get_app_name(service_name.clone());
        let path = self.root_path.clone() + "/" + &app_name;
        if self.listeners.read().unwrap().get(service_name).is_some() {
            return Ok(());
        }

        let arc_listener = Arc::new(listener);
        self.listeners
            .write()
            .unwrap()
            .insert(service_name.to_string(), Arc::clone(&arc_listener));

        let zk_listener = self.create_listener(
            path.clone(),
            service_name.to_string(),
            Arc::clone(&arc_listener),
        );

        let res = self.zk_client.get_children_w(&path, zk_listener);
        let result: Vec<Url> = res
            .unwrap()
            .iter()
            .map(|node_key| {
                let zk_res = self.zk_client.get_data(
                    &(self.root_path.clone() + "/" + &app_name + "/" + &node_key),
                    false,
                );
                let vec_u8 = zk_res.unwrap().0;
                let sstr = std::str::from_utf8(&vec_u8).unwrap();
                let instance: ZkServiceInstance = serde_json::from_str(sstr).unwrap();
                let url = Url::from_url(&format!(
                    "triple://{}:{}/{}",
                    instance.get_host(),
                    instance.get_port(),
                    service_name
                ))
                .unwrap();
                url
            })
            .collect();

        info!("notifing {}->{:?}", service_name, result);
        arc_listener.notify(ServiceEvent {
            key: service_name.to_string(),
            action: String::from("ADD"),
            service: result,
        });
        Ok(())
    }

    fn unsubscribe(&self, url: Url, listener: Self::NotifyListener) -> Result<(), StdError> {
        todo!()
    }
}

pub struct ServiceInstancesChangedListener {
    zk_client: Arc<ZooKeeper>,
    path: String,

    service_name: String,
    listener: Arc<MemoryNotifyListener>,
}

impl Watcher for ServiceInstancesChangedListener {
    fn handle(&self, event: WatchedEvent) {
        if let (WatchedEventType::NodeChildrenChanged, Some(path)) = (event.event_type, event.path)
        {
            let event_path = path.clone();
            let dirs = self
                .zk_client
                .get_children(&event_path.clone(), false)
                .expect("msg");
            let result: Vec<Url> = dirs
                .iter()
                .map(|node_key| {
                    let zk_res = self
                        .zk_client
                        .get_data(&(event_path.clone() + "/" + node_key), false);
                    let vec_u8 = zk_res.unwrap().0;
                    let sstr = std::str::from_utf8(&vec_u8).unwrap();
                    let instance: ZkServiceInstance = serde_json::from_str(sstr).unwrap();
                    let url = Url::from_url(&format!(
                        "triple://{}:{}/{}",
                        instance.get_host(),
                        instance.get_port(),
                        self.service_name
                    ))
                    .unwrap();
                    url
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

            info!("notifing {}->{:?}", self.service_name, result);
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
        todo!()
    }

    fn notify_all(&self, event: ServiceEvent) {
        todo!()
    }
}

impl ClusterRegistryIntegration for ZookeeperRegistry {
    fn get_invoker(registry: BoxRegistry) -> Option<Arc<ClusterInvoker>> {
        todo!()
    }
}

#[test]
fn it_works() {
    let connect_string = &"mse-21b397d4-p.zk.mse.aliyuncs.com:2181";
    let zk_client =
        ZooKeeper::connect(connect_string, Duration::from_secs(15), LoggingWatcher).unwrap();
    let watcher = Arc::new(Some(TestZkWatcher {
        watcher: Arc::new(None),
    }));
    watcher.as_ref().expect("").watcher = Arc::clone(&watcher);
    let x = watcher.as_ref().expect("");
    zk_client.get_children_w("/test", x);
    zk_client.delete("/test/a", None);
    zk_client.delete("/test/b", None);
    let zk_res = zk_client.create(
        "/test/a",
        vec![1, 3],
        Acl::open_unsafe().clone(),
        CreateMode::Ephemeral,
    );
    let zk_res = zk_client.create(
        "/test/b",
        vec![1, 3],
        Acl::open_unsafe().clone(),
        CreateMode::Ephemeral,
    );
    zk_client.close();
}

struct TestZkWatcher {
    pub watcher: Arc<Option<TestZkWatcher>>,
}

impl Watcher for TestZkWatcher {
    fn handle(&self, event: WatchedEvent) {
        println!("event: {:?}", event);
    }
}

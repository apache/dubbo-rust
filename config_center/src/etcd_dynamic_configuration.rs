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

use std::collections::{HashMap, HashSet};
use etcd_client::{Client, GetOptions, Watcher, WatchOptions, WatchStream};
use crate::configuration_listener::ConfigurationListener;
use crate::dynamic_configuration::DynamicConfiguration;
use async_trait::async_trait;
use rand::Rng;
use crate::url::URL;

pub struct EtcdDynamicConfiguration {

    /**
     * The final root path would be: /$NAME_SPACE/config
     */
    pub root_path: String,

    pub client: Client,

    pub url: URL,

    pub watcher: Watcher,

    pub stream: WatchStream,

    pub watch_listener_map: HashMap<String, HashMap<String, EtcdConfigWatcher>>,
}

const CONFIG_NAMESPACE_KEY: &str = "namespace";

const DEFAULT_GROUP: &str = "dubbo";

const PATH_SEPARATOR: &str = "/";

impl EtcdDynamicConfiguration {

    pub async fn new(self, url: URL) -> Self {
        let mut client = Client::connect(["localhost:2379"], None).await.unwrap();
        let mut root_path = String::from(PATH_SEPARATOR);
        root_path.push_str(url.get_parameter(CONFIG_NAMESPACE_KEY, DEFAULT_GROUP).as_str());
        root_path.push_str("/config");
        let (watcher, stream) = client.watch("/", None).await.unwrap();
        let watch_listener_map = HashMap::new();


        // while let Some(resp) = stream.message().await? {
        //     println!("[{:?}] receive watch response", resp.watch_id());
        //     for event in resp.events() {
        //         println!("event type: {:?}", event.event_type());
        //         if let Some(kv) = event.kv() {
        //             println!("kv: {{{}: {}}}", kv.key_str()?, kv.value_str()?);
        //         }
        //         if EventType::Delete == event.event_type() {
        //             watcher.cancel_by_id(resp.watch_id()).await?;
        //         }
        //     }
        // }




        EtcdDynamicConfiguration {
            root_path,
            client,
            url,
            watcher,
            stream,
            watch_listener_map,
        }
    }

    pub fn get_path(&self, key: &str, group: &str) -> String {
        if key.is_empty() {
            return self.build_path(group);
        }
        self.build_path(group) + PATH_SEPARATOR + key
    }

    pub fn build_path(&self, mut group: &str) -> String {
        if group.is_empty() {
            group = DEFAULT_GROUP;
        }
        self.root_path.clone() + PATH_SEPARATOR + group
    }
}

#[async_trait]
impl DynamicConfiguration for EtcdDynamicConfiguration {

    async fn add_listener(mut self, key: &str, group: &str, listener: impl ConfigurationListener  + std::marker::Send) {
        let path = self.get_path(key, group);
        let watch_id = rand::thread_rng().gen();
        if !self.watch_listener_map.contains_key(path.as_str()) {
            let mut watcher_map = HashMap::new();
            let listener_type = listener.get_type();
            let mut etcd_watcher = EtcdConfigWatcher::new(key.to_string(), group.to_string(), self.watcher, self.stream, watch_id, listener);
            etcd_watcher.watch(watch_id);
            watcher_map.insert(listener_type, etcd_watcher);
            self.watch_listener_map.insert(path, watcher_map);
        } else {
            let watcher_map = self.watch_listener_map.get_mut(path.as_str()).unwrap();
            let listener_type = listener.get_type();
            let mut etcd_watcher = EtcdConfigWatcher::new(key.to_string(), group.to_string(), self.watcher, self.stream, watch_id, listener);
            etcd_watcher.watch(watch_id);
            watcher_map.insert(listener_type, etcd_watcher);
        }
    }

    async fn remove_listener(mut self, key: &str, group: &str, listener: impl ConfigurationListener  + std::marker::Send) {
        let path = self.get_path(key, group);
        let watcher_map = self.watch_listener_map.get_mut(path.as_str()).unwrap();
        if !watcher_map.contains_key(listener.get_type().as_str()) {
            return;
        }
        let watcher = watcher_map.get_mut(listener.get_type().as_str()).unwrap();
        watcher.cancelWatch();
        watcher_map.remove(listener.get_type().as_str());
    }

    async fn get_config(&mut self, key: &str, group: &str, timeout: i32) -> String {
        let path = self.get_path(key, group);
        let resp = self.client.get(path, None).await.unwrap();
        if let Some(kv) = resp.kvs().first() {
            return kv.value_str().unwrap().to_string();
        }
        return String::new();
    }

    async fn get_properties(&mut self, key: &str, group: &str, timeout: i32) -> String {
        let mut path = String::new();
        if group.len() != 0 {
            path = group.to_string() + PATH_SEPARATOR + key;
        } else {
            path = self.url.get_parameter(CONFIG_NAMESPACE_KEY, DEFAULT_GROUP) + PATH_SEPARATOR + key;
        }
        let resp = self.client.get(key, None).await.unwrap();
        if let Some(kv) = resp.kvs().first() {
            return kv.value_str().unwrap().to_string();
        }
        return String::from("");
    }

    async fn publish_config(&mut self, key: &str, group: &str, content: &str) -> bool {
        let path = self.get_path(key, group);

        // TODO need base64 encoding

        self.client.put(path, content, None).await.unwrap();

        // TODO consider fix return value type.
        true
    }

    async fn get_config_keys(&mut self, group: &str) -> HashSet<String> {
        let path = self.get_path("", group);
        let resp = self.client.get("", Some(GetOptions::new().with_prefix())).await.unwrap();
        let mut result = HashSet::new();
        for kv in resp.kvs() {
            result.insert(kv.key_str().unwrap().to_string());
        }
        result
    }
}

pub struct EtcdConfigWatcher {
    //pub listener: Box<dyn ConfigurationListener>,
    pub key: String,
    pub group: String,
    pub normalized_key: String,
    pub watcher: Watcher,
    pub stream: WatchStream,
    pub watch_id: i64,
}

impl EtcdConfigWatcher {

    pub fn new(key: String, group: String, watcher: Watcher, stream: WatchStream, watch_id: i64, listener: impl ConfigurationListener) -> Self {
        EtcdConfigWatcher {
            //listener,
            key,
            group,
            normalized_key: "".to_string(), // TODO
            watcher,
            stream,
            watch_id,
        }
    }

    pub async fn watch(&mut self, watch_id: i64) {
        self.watcher.watch(self.key.clone(), Some(WatchOptions::new().with_watch_id(watch_id))).await.unwrap();
    }

    pub fn cancelWatch(&mut self) {
        let watch_id = self.watch_id;
        self.watcher.cancel_by_id(watch_id);
    }
}

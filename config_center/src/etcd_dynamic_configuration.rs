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
use etcd_client::Client;
use crate::configuration_listener::ConfigurationListener;
use crate::dynamic_configuration::DynamicConfiguration;
use crate::url::URL;
use async_trait::async_trait;

pub struct EtcdDynamicConfiguration {

    /**
     * The final root path would be: /$NAME_SPACE/config
     */
    pub root_path: String,

    pub client: Client,
}

const CONFIG_NAMESPACE_KEY: &str = "namespace";

const DEFAULT_GROUP: &str = "dubbo";

impl EtcdDynamicConfiguration {

    pub async fn new(&self, url: URL) -> Self {
        let client = Client::connect(["localhost:2379"], None).await.unwrap();
        let mut root_path = String::from("/");
        root_path.push_str(url.get_parameter(String::from(CONFIG_NAMESPACE_KEY), String::from(DEFAULT_GROUP)).as_str());
        root_path.push_str("/config");
        EtcdDynamicConfiguration {
            root_path,
            client,
        }
    }
}

#[async_trait]
impl DynamicConfiguration for EtcdDynamicConfiguration {
    fn add_listener(&self, key: String, listener: impl ConfigurationListener) {
        todo!()
    }

    fn remove_listener(&self, key: String, listener: impl ConfigurationListener) {
        todo!()
    }

    async fn get_config(&mut self, key: String, group: String, timeout: i32) -> String {
        if key.is_empty() {
            return String::from("");
        }
        let resp = self.client.get(key, None).await.unwrap();
        if let Some(kv) = resp.kvs().first() {
            return kv.value_str().unwrap().to_string();
        }
        return String::from("");
    }

    fn get_properties(&self, key: String, group: String, timeout: i32) -> String {
        todo!()
    }

    fn publish_config(&self, key: String, group: String, content: String) -> bool {
        todo!()
    }

    fn get_config_keys(&self, group: String) -> HashMap<String, String> {
        todo!()
    }
}
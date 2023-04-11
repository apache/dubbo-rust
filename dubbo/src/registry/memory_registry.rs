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

use dubbo_logger::tracing::debug;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use dubbo_base::Url;

use super::{NotifyListener, Registry, RegistryNotifyListener};

// 从url中获取服务注册的元数据
/// rawURL = fmt.Sprintf("%s://%s%s?%s", c.Protocol, host, c.Path, s)
/// dubboPath = fmt.Sprintf("/%s/%s/%s", r.URL.GetParam(constant.RegistryGroupKey, "dubbo"), r.service(c), common.DubboNodes[common.PROVIDER])

pub const REGISTRY_GROUP_KEY: &str = "registry.group";

#[derive(Debug, Default, Clone)]
pub struct MemoryRegistry {
    registries: Arc<RwLock<HashMap<String, String>>>,
}

impl MemoryRegistry {
    pub fn new() -> MemoryRegistry {
        MemoryRegistry {
            registries: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Registry for MemoryRegistry {
    fn register(&mut self, mut url: Url) -> Result<(), crate::StdError> {
        // define provider label: ${registry.group}/${service_name}/provider
        let registry_group = match url.get_param(REGISTRY_GROUP_KEY) {
            Some(key) => key,
            None => "dubbo".to_string(),
        };

        let dubbo_path = format!(
            "/{}/{}/{}",
            registry_group,
            url.get_service_name(),
            "provider",
        );

        url.params.insert("anyhost".to_string(), "true".to_string());
        // define triple url path
        let raw_url = url.raw_url_string();

        self.registries.write().unwrap().insert(dubbo_path, raw_url);
        Ok(())
    }

    fn unregister(&mut self, url: dubbo_base::Url) -> Result<(), crate::StdError> {
        let registry_group = match url.get_param(REGISTRY_GROUP_KEY) {
            Some(key) => key,
            None => "dubbo".to_string(),
        };

        let dubbo_path = format!(
            "/{}/{}/{}",
            registry_group,
            url.get_service_name(),
            "provider",
        );
        self.registries.write().unwrap().remove(&dubbo_path);

        Ok(())
    }

    fn subscribe(
        &self,
        url: dubbo_base::Url,
        listener: RegistryNotifyListener,
    ) -> Result<(), crate::StdError> {
        todo!()
    }

    fn unsubscribe(
        &self,
        url: dubbo_base::Url,
        listener: RegistryNotifyListener,
    ) -> Result<(), crate::StdError> {
        todo!()
    }
}

pub struct MemoryNotifyListener {
    pub service_instances: Arc<RwLock<HashMap<String, Vec<Url>>>>,
}

impl NotifyListener for MemoryNotifyListener {
    fn notify(&self, event: super::ServiceEvent) {
        debug!("notify {:?}", event);
        let mut map = self.service_instances.write().expect("msg");
        match event.action.as_str() {
            "ADD" => map.insert(event.key, event.service),
            &_ => todo!(),
        };
    }

    fn notify_all(&self, event: super::ServiceEvent) {
        todo!()
    }
}

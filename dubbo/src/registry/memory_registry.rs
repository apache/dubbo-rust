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
use std::collections::HashMap;
use std::sync::RwLock;

use super::{NotifyListener, Registry};

// 从url中获取服务注册的元数据
/// rawURL = fmt.Sprintf("%s://%s%s?%s", c.Protocol, host, c.Path, s)
/// dubboPath = fmt.Sprintf("/%s/%s/%s", r.URL.GetParam(constant.RegistryGroupKey, "dubbo"), r.service(c), common.DubboNodes[common.PROVIDER])

pub const REGISTRY_GROUP_KEY: &str = "registry.group";

#[derive(Debug, Default)]
pub struct MemoryRegistry {
    registries: RwLock<HashMap<String, String>>,
}

impl MemoryRegistry {
    pub fn new() -> MemoryRegistry {
        MemoryRegistry {
            registries: RwLock::new(HashMap::new()),
        }
    }
}

impl Registry for MemoryRegistry {
    type NotifyListener = MemoryNotifyListener;

    fn register(&mut self, mut url: crate::common::url::Url) -> Result<(), crate::StdError> {
        // define provider label: ${registry.group}/${service_name}/provider
        let registry_group = match url.get_param(REGISTRY_GROUP_KEY.to_string()) {
            Some(key) => key,
            None => "dubbo".to_string(),
        };

        let dubbo_path = format!(
            "/{}/{}/{}",
            registry_group,
            url.get_service_name().join(","),
            "provider",
        );

        url.params.insert("anyhost".to_string(), "true".to_string());
        // define triple url path
        let raw_url = format!("{}?{}", url.to_url(), url.encode_param(),);

        self.registries.write().unwrap().insert(dubbo_path, raw_url);
        Ok(())
    }

    fn unregister(&mut self, url: crate::common::url::Url) -> Result<(), crate::StdError> {
        let registry_group = match url.get_param(REGISTRY_GROUP_KEY.to_string()) {
            Some(key) => key,
            None => "dubbo".to_string(),
        };

        let dubbo_path = format!(
            "/{}/{}/{}",
            registry_group,
            url.get_service_name().join(","),
            "provider",
        );
        self.registries.write().unwrap().remove(&dubbo_path);

        Ok(())
    }

    fn subscribe(
        &self,
        url: crate::common::url::Url,
        listener: Self::NotifyListener,
    ) -> Result<(), crate::StdError> {
        todo!()
    }

    fn unsubscribe(
        &self,
        url: crate::common::url::Url,
        listener: Self::NotifyListener,
    ) -> Result<(), crate::StdError> {
        todo!()
    }
}

pub struct MemoryNotifyListener {}

impl NotifyListener for MemoryNotifyListener {
    fn notify(&self, event: super::ServiceEvent) {
        todo!()
    }

    fn notify_all(&self, event: super::ServiceEvent) {
        todo!()
    }
}

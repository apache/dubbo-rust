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

use std::{
    collections::HashMap,
    fmt::Debug,
    str::FromStr,
    sync::{Arc, RwLock},
};

use crate::{
    common::url::Url,
    invocation::{Invocation, RpcInvocation},
    registry::{memory_registry::MemoryNotifyListener, BoxRegistry, RegistryWrapper},
};

/// Directory.
///
/// [Directory Service](http://en.wikipedia.org/wiki/Directory_service)
pub trait Directory: Debug + DirectoryClone {
    fn list(&self, invocation: RpcInvocation) -> Vec<Url>;
}

pub trait DirectoryClone {
    fn clone_box(&self) -> Box<dyn Directory>;
}

impl<T> DirectoryClone for T
where
    T: 'static + Directory + Clone,
{
    fn clone_box(&self) -> Box<dyn Directory> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Directory> {
    fn clone(&self) -> Box<dyn Directory> {
        self.clone_box()
    }
}

#[derive(Debug)]
pub struct StaticDirectory {
    uri: http::Uri,
}

impl StaticDirectory {
    pub fn new(host: &str) -> StaticDirectory {
        let uri = match http::Uri::from_str(host) {
            Ok(v) => v,
            Err(err) => {
                tracing::error!("http uri parse error: {}, host: {}", err, host);
                panic!("http uri parse error: {}, host: {}", err, host)
            }
        };
        StaticDirectory { uri: uri }
    }

    pub fn from_uri(uri: &http::Uri) -> StaticDirectory {
        StaticDirectory { uri: uri.clone() }
    }
}

impl Directory for StaticDirectory {
    fn list(&self, invocation: RpcInvocation) -> Vec<Url> {
        let url = Url::from_url(&format!(
            "triple://{}:{}/{}",
            self.uri.host().unwrap(),
            self.uri.port().unwrap(),
            invocation.get_target_service_unique_name(),
        ))
        .unwrap();
        vec![url]
    }
}

impl DirectoryClone for StaticDirectory {
    fn clone_box(&self) -> Box<dyn Directory> {
        Box::new(StaticDirectory {
            uri: self.uri.clone(),
        })
    }
}

#[derive(Debug)]
pub struct RegistryDirectory {
    registry: RegistryWrapper,
    service_instances: Arc<RwLock<HashMap<String, Vec<Url>>>>,
}

impl RegistryDirectory {
    pub fn new(registry: BoxRegistry) -> RegistryDirectory {
        RegistryDirectory {
            registry: RegistryWrapper {
                registry: Some(registry),
            },
            service_instances: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl DirectoryClone for RegistryDirectory {
    fn clone_box(&self) -> Box<dyn Directory> {
        todo!()
    }
}

impl Directory for RegistryDirectory {
    fn list(&self, invocation: RpcInvocation) -> Vec<Url> {
        let service_name = invocation.get_target_service_unique_name();

        let url = Url::from_url(&format!(
            "triple://{}:{}/{}",
            "127.0.0.1", "8888", service_name
        ))
        .unwrap();

        self.registry
            .registry
            .as_ref()
            .expect("msg")
            .subscribe(
                url,
                MemoryNotifyListener {
                    service_instances: Arc::clone(&self.service_instances),
                },
            )
            .expect("subscribe");

        let map = self
            .service_instances
            .read()
            .expect("service_instances.read");
        let binding = Vec::new();
        let url_vec = map.get(&service_name).unwrap_or(&binding);
        url_vec.to_vec()
    }
}

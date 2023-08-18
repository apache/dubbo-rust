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

use std::{collections::HashMap, env, path::PathBuf};

use crate::{protocol::Protocol, registry::RegistryConfig};
use dubbo_logger::tracing;
use dubbo_utils::yaml_util::yaml_file_parser;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

use super::{protocol::ProtocolConfig, provider::ProviderConfig, service::ServiceConfig};

pub const DUBBO_CONFIG_PATH: &str = "application.yaml";

pub static GLOBAL_ROOT_CONFIG: OnceCell<RootConfig> = OnceCell::new();
pub const DUBBO_CONFIG_PREFIX: &str = "dubbo";

/// used to storage all structed config, from some source: cmd, file..;
/// Impl Config trait, business init by read Config trait
#[allow(dead_code)]
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct RootConfig {
    #[serde(default)]
    pub protocols: ProtocolConfig,

    #[serde(default)]
    pub provider: ProviderConfig,

    #[serde(default)]
    pub registries: HashMap<String, RegistryConfig>,

    #[serde(default)]
    pub data: HashMap<String, String>,
}

pub fn get_global_config() -> &'static RootConfig {
    GLOBAL_ROOT_CONFIG.get_or_init(|| {
        tracing::debug!("current path: {:?}", env::current_dir());
        RootConfig::new()
            .load()
            .unwrap_or_else(|err| panic!("Failed to load global config, error: {}", err))
    })
}

impl RootConfig {
    pub fn new() -> Self {
        Self {
            protocols: HashMap::new(),
            registries: HashMap::new(),
            provider: ProviderConfig::new(),
            data: HashMap::new(),
        }
    }

    pub fn load(&self) -> std::io::Result<Self> {
        let config_path = match env::var("DUBBO_CONFIG_PATH") {
            Ok(v) => {
                tracing::info!("read config_path from env: {:?}", v);
                v
            }
            Err(err) => {
                tracing::warn!(
                    "error loading config_path: {:?}, use default path: {:?}",
                    err,
                    DUBBO_CONFIG_PATH
                );
                dubbo_utils::path_util::app_root_dir()
                    .join(DUBBO_CONFIG_PATH)
                    .to_str()
                    .unwrap()
                    .to_string()
            }
        };

        let conf: HashMap<String, RootConfig> =
            yaml_file_parser(PathBuf::new().join(config_path)).unwrap();
        let root_config: RootConfig = conf.get(DUBBO_CONFIG_PREFIX).unwrap().clone();
        tracing::debug!("origin config: {:?}", conf);
        Ok(root_config)
    }

    pub fn test_config(&mut self) {
        let mut provider = ProviderConfig::new();
        provider.protocol_ids = vec!["triple".to_string()];
        provider.registry_ids = vec![];

        let service_config = ServiceConfig::default()
            .group("test".to_string())
            .version("1.0.0".to_string())
            .protocol("triple".to_string())
            .interface("grpc.examples.echo.Echo".to_string());

        self.provider
            .services
            .insert("grpc.examples.echo.Echo".to_string(), service_config);
        self.provider.services.insert(
            "helloworld.Greeter".to_string(),
            ServiceConfig::default()
                .group("test".to_string())
                .version("1.0.0".to_string())
                .interface("helloworld.Greeter".to_string())
                .protocol("triple".to_string()),
        );
        self.protocols.insert(
            "triple".to_string(),
            Protocol::default()
                .name("triple".to_string())
                .ip("0.0.0.0".to_string())
                .port("8889".to_string()),
        );

        self.provider = provider.clone();
        println!("provider config: {:?}", provider);
        // 通过环境变量读取某个文件。加在到内存中
        self.data.insert(
            "dubbo.provider.url".to_string(),
            "dubbo://127.0.0.1:8888/?serviceName=hellworld".to_string(),
        );
        // self.data.insert("dubbo.consume.", v)
    }

    #[inline]
    pub fn leak(self) -> &'static Self {
        Box::leak(Box::new(self))
    }
}

impl Config for RootConfig {
    fn bool(&self, key: String) -> bool {
        match self.data.get(&key) {
            None => false,
            Some(val) => match val.parse::<bool>() {
                Ok(v) => v,
                Err(_err) => {
                    tracing::error!("key: {}, val: {} is not boolean", key, val);
                    false
                }
            },
        }
    }

    fn string(&self, key: String) -> String {
        match self.data.get(&key) {
            None => "".to_string(),
            Some(val) => val.to_string(),
        }
    }
}

pub trait BusinessConfig {
    fn init() -> Self;
    fn load() -> Result<(), std::convert::Infallible>;
}

pub trait Config {
    fn bool(&self, key: String) -> bool;
    fn string(&self, key: String) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        let mut r = RootConfig::new();
        r.test_config();
    }

    #[test]
    fn test_load() {
        dubbo_logger::init();
        let r = RootConfig::new();
        let r = r.load().unwrap();
        println!("{:#?}", r);
    }
}

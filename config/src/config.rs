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

use std::{any, collections::HashMap, env, fs};

use serde::{Deserialize, Serialize};

use super::protocol::ProtocolConfig;
use super::service::ServiceConfig;

pub const DUBBO_CONFIG_PATH: &str = "./dubbo.yaml";

/// used to storage all structed config, from some source: cmd, file..;
/// Impl Config trait, business init by read Config trait
#[allow(dead_code)]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RootConfig {
    pub name: String,
    pub service: HashMap<String, ServiceConfig>,
    pub protocols: HashMap<String, ProtocolConfig>,

    #[serde(skip_serializing, skip_deserializing)]
    pub data: HashMap<String, Box<dyn any::Any>>,
}

pub fn get_global_config() -> RootConfig {
    tracing::debug!("current path: {:?}", env::current_dir());
    let c = match RootConfig::new().load() {
        Ok(v) => v,
        Err(err) => panic!("Error loading global config, error: {}", err),
    };
    c
}

impl RootConfig {
    pub fn new() -> Self {
        Self {
            name: "dubbo".to_string(),
            service: HashMap::new(),
            protocols: HashMap::new(),
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
                tracing::error!(
                    "error loading config_path: {:?}, use default path: {:?}",
                    err,
                    DUBBO_CONFIG_PATH
                );
                DUBBO_CONFIG_PATH.to_string()
            }
        };

        let data = fs::read(config_path)?;
        let mut conf: RootConfig = serde_yaml::from_slice(&data).unwrap();
        tracing::debug!("origin config: {:?}", conf);
        for (name, svc) in conf.service.iter_mut() {
            svc.name = name.to_string();
        }
        Ok(conf)
    }

    pub fn test_config(&mut self) {
        let service_config = ServiceConfig::default()
            .group("test".to_string())
            .serializer("json".to_string())
            .version("1.0.0".to_string())
            .protocol_names("triple".to_string())
            .name("echo".to_string());

        let triple_config = ProtocolConfig::default()
            .name("triple".to_string())
            .ip("0.0.0.0".to_string())
            .port("8888".to_string());

        let service_config = service_config.add_protocol_configs(triple_config);
        self.service.insert("echo".to_string(), service_config);
        self.service.insert(
            "helloworld.Greeter".to_string(),
            ServiceConfig::default()
                .group("test".to_string())
                .serializer("json".to_string())
                .version("1.0.0".to_string())
                .name("helloworld.Greeter".to_string())
                .protocol_names("triple".to_string()),
        );
        self.protocols.insert(
            "triple".to_string(),
            ProtocolConfig::default()
                .name("triple".to_string())
                .ip("0.0.0.0".to_string())
                .port("8889".to_string()),
        );
        // 通过环境变量读取某个文件。加在到内存中
        self.data.insert(
            "dubbo.provider.url".to_string(),
            Box::new("dubbo://127.0.0.1:8888/?serviceName=hellworld".to_string()),
        );
        // self.data.insert("dubbo.consume.", v)
    }
}

impl Config for RootConfig {
    fn bool(&self, key: String) -> bool {
        match self.data.get(&key) {
            None => false,
            Some(val) => {
                if let Some(v) = val.downcast_ref::<bool>() {
                    *v
                } else {
                    false
                }
            }
        }
    }

    fn string(&self, key: String) -> String {
        match self.data.get(&key) {
            None => "".to_string(),
            Some(val) => {
                if let Some(v) = val.downcast_ref::<String>() {
                    v.into()
                } else {
                    "".to_string()
                }
            }
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
    fn test_load() {
        // case 1: read config yaml from default path
        println!("current path: {:?}", env::current_dir());
        let r = RootConfig::new();
        r.load();
    }

    #[test]
    fn test_load1() {
        // case 2: read config yaml from path in env
        println!("current path: {:?}", env::current_dir());
        let r = RootConfig::new();
        r.load();
    }

    #[test]
    fn test_write_yaml() {
        let mut r = RootConfig::new();
        r.test_config();
        let yaml = serde_yaml::to_string(&r).unwrap();
        println!("config data: {:?}", yaml);

        fs::write("./test_dubbo.yaml", yaml);
    }
}

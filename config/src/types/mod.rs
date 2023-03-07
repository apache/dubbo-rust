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

use crate::get_config_location;
use crate::types::consumer::ConsumerConfig;
use crate::types::protocol::ProtocolConfig;
use crate::types::provider::ProviderConfig;
use crate::types::registry::RegistryConfig;
use crate::types::services::ServicesConfig;
use crate::util::yaml_file_parser;
use anyhow::Error;
use base::constants::DUBBO_KEY;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub mod consumer;
pub mod protocol;
pub mod provider;
pub mod registry;
pub mod services;

/// used to storage all structed config, from some source: cmd, file..;
/// Impl Config trait, business init by read Config trait
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone, Getters, Setters, MutGetters, CopyGetters)]
pub struct RootConfig {
    #[serde(default)]
    pub location: PathBuf,

    #[serde(default)]
    #[getset(get, set, get_mut)]
    pub protocols: ProtocolConfig,

    #[serde(default)]
    #[getset(get, set, get_mut)]
    pub provider: ProviderConfig,

    #[serde(default)]
    #[getset(get, set, get_mut)]
    pub registries: RegistryConfig,

    #[serde(default)]
    #[getset(get, set, get_mut)]
    pub consumer: ConsumerConfig,

    #[serde(default)]
    #[getset(get, set, get_mut)]
    pub services: ServicesConfig,
}

impl Default for RootConfig {
    fn default() -> RootConfig {
        let conf: HashMap<String, RootConfig> = yaml_file_parser(get_config_location()).unwrap();
        let mut root_config: RootConfig = conf.get(DUBBO_KEY).unwrap().clone();
        root_config.location = get_config_location();
        root_config
    }
}

#[derive(Clone)]
pub struct ConfigWrapper {
    pub inner: Arc<Mutex<RootConfig>>,
}

impl ConfigWrapper {
    pub fn new(inner: Arc<Mutex<RootConfig>>) -> Self {
        ConfigWrapper { inner }
    }
}

pub trait ConfigValidator {
    fn validate(&self) -> Result<(), Error>;
}

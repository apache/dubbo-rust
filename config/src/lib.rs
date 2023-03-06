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

use base::constants::DUBBO_KEY;
use base::types::alias::RegistryId;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod location;
pub mod types;
pub mod util;

use crate::types::consumer::ConsumerConfig;
use crate::types::protocol::ProtocolConfig;
use crate::types::provider::ProviderConfig;
use crate::types::registry::RegistryConfig;
use crate::util::yaml_file_parser;
pub use location::get_config_location;

pub type ConfigWrapper = Arc<Mutex<RootConfig>>;

static GLOBAL_ROOT_CONFIG: Lazy<ConfigWrapper> =
    Lazy::new(|| Arc::new(Mutex::new(RootConfig::default())));

/// used to storage all structed config, from some source: cmd, file..;
/// Impl Config trait, business init by read Config trait
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RootConfig {
    #[serde(default)]
    pub protocols: ProtocolConfig,

    #[serde(default)]
    pub provider: ProviderConfig,

    #[serde(default)]
    pub registries: HashMap<RegistryId, RegistryConfig>,

    pub consumers: HashMap<String, ConsumerConfig>,
}

impl Default for RootConfig {
    fn default() -> RootConfig {
        let conf: HashMap<String, RootConfig> = yaml_file_parser(get_config_location()).unwrap();
        let root_config: RootConfig = conf.get(DUBBO_KEY).unwrap().clone();
        root_config
    }
}

pub fn get_root_config() -> ConfigWrapper {
    GLOBAL_ROOT_CONFIG.clone()
}

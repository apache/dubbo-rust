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

use serde::Deserialize;
use serde::Serialize;

use base::types::alias::{
    ClusterStrategy, FilterKey, GroupId, InterfaceName, ParamKey, ProtocolKey, RegistryId,
    ServiceName,
};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ConsumerConfig {
    #[serde(default)]
    pub registry_ids: Vec<RegistryId>,
    #[serde(default)]
    pub filter: FilterKey,
    #[serde(default)]
    pub protocol_ids: Vec<String>,
    #[serde(default)]
    pub references: ReferenceConfig,
}

pub type ReferenceConfig = HashMap<ServiceName, Reference>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Reference {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub protocol: ProtocolKey,
    #[serde(default)]
    pub group: GroupId,
    #[serde(default)]
    pub interface_name: InterfaceName,
    #[serde(default)]
    pub registry_ids: Vec<RegistryId>,
    #[serde(default)]
    pub cluster: ClusterStrategy,
    #[serde(default)]
    pub params: HashMap<ParamKey, String>,
    #[serde(default)]
    pub retries: u8,
}

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

use crate::types::default::*;
use crate::types::ConfigValidator;
use anyhow::Error;
use base::types::alias::{
    GroupId, InterfaceName, ProtocolId, SerializationKey, ServiceName, VersionNumber,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type ServicesConfig = HashMap<ServiceName, Service>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Service {
    #[serde(default = "default_version_number")]
    pub version: VersionNumber,
    #[serde(default = "default_group_id")]
    pub group: GroupId,
    #[serde(default)]
    pub protocol: ProtocolId,
    #[serde(default)]
    pub interface: InterfaceName,
    #[serde(default)]
    pub serialization: SerializationKey,
}

impl ConfigValidator for Service {
    fn validate(&self) -> Result<(), Error> {
        todo!()
    }
}

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
use anyhow::Error;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::ConfigValidator;
use base::types::alias::{ParamKey, RegistryId, RegistryType};
pub type RegistryConfig = HashMap<RegistryId, Registry>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Registry {
    #[serde(default)]
    pub protocol: RegistryId,
    #[serde(default)]
    pub timeout: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub params: HashMap<ParamKey, String>,
    #[serde(default)]
    pub registry_type: RegistryType,
}

impl ConfigValidator for Registry {
    fn validate(&self) -> Result<(), Error> {
        todo!()
    }
}

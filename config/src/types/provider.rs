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

use serde::{Deserialize, Serialize};

use super::service::ServiceConfig;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ProviderConfig {
    #[serde(default)]
    pub registry_ids: Vec<String>,
    #[serde(default)]
    pub protocol_ids: Vec<String>,
    #[serde(default)]
    pub services: HashMap<String, ServiceConfig>,
}

impl ProviderConfig {
    pub fn new() -> Self {
        ProviderConfig {
            registry_ids: vec![],
            protocol_ids: vec![],
            services: HashMap::new(),
        }
    }

    pub fn with_registry_ids(mut self, registry_ids: Vec<String>) -> Self {
        self.registry_ids = registry_ids;
        self
    }

    pub fn with_protocol_ids(mut self, protocol_ids: Vec<String>) -> Self {
        self.protocol_ids = protocol_ids;
        self
    }

    pub fn with_services(mut self, services: HashMap<String, ServiceConfig>) -> Self {
        self.services = services;
        self
    }
}

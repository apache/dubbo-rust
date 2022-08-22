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

use super::protocol::ProtocolConfig;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ServiceConfig {
    pub version: String,
    pub group: String,

    #[serde(skip_serializing, skip_deserializing)]
    pub name: String,
    pub protocol: String,
    pub registry: String,
    pub serializer: String,

    // #[serde(skip_serializing, skip_deserializing)]
    pub protocol_configs: HashMap<String, ProtocolConfig>,
}

impl ServiceConfig {
    pub fn name(self, name: String) -> Self {
        Self { name, ..self }
    }

    pub fn version(self, version: String) -> Self {
        Self { version, ..self }
    }

    pub fn group(self, group: String) -> Self {
        Self { group, ..self }
    }

    pub fn protocol_names(self, protocol: String) -> Self {
        Self { protocol, ..self }
    }

    pub fn serializer(self, serializer: String) -> Self {
        Self { serializer, ..self }
    }

    pub fn add_protocol_configs(mut self, protocol_config: ProtocolConfig) -> Self {
        self.protocol_configs
            .insert(protocol_config.name.clone(), protocol_config);
        Self { ..self }
    }

    // pub fn get_url(&self) -> Vec<Url> {
    //     let mut urls = Vec::new();
    //     for (_, conf) in self.protocol_configs.iter() {
    //         urls.push(Url {
    //             url: conf.to_owned().to_url(),
    //             service_key: "".to_string(),
    //         });
    //     }

    //     urls
    // }
}

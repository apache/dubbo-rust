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

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    pub ip: String,
    pub port: String,
    pub name: String,
    pub listener: String,

    #[serde(skip_serializing, skip_deserializing)]
    pub params: HashMap<String, String>,
}

impl ProtocolConfig {
    pub fn name(self, name: String) -> Self {
        Self { name, ..self }
    }

    pub fn ip(self, ip: String) -> Self {
        Self { ip, ..self }
    }

    pub fn port(self, port: String) -> Self {
        Self { port, ..self }
    }

    pub fn listener(self, listener: String) -> Self {
        Self { listener, ..self }
    }

    pub fn params(self, params: HashMap<String, String>) -> Self {
        Self { params, ..self }
    }

    pub fn add_param(mut self, key: String, value: String) -> Self {
        self.params.insert(key, value);
        self
    }

    pub fn to_url(&self) -> String {
        let mut params_vec: Vec<String> = Vec::new();
        for (k, v) in self.params.iter() {
            // let tmp = format!("{}={}", k, v);
            params_vec.push(format!("{}={}", k, v));
        }
        let param = params_vec.join("&");

        format!(
            "{}://{}:{}?listener={}&{}",
            self.name, self.ip, self.port, self.listener, param
        )
    }
}

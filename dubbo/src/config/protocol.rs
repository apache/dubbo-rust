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

pub const DEFAULT_PROTOCOL: &str = "triple";

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Protocol {
    pub ip: String,
    pub port: String,
    pub name: String,

    #[serde(skip_serializing, skip_deserializing)]
    pub params: HashMap<String, String>,
}

pub type ProtocolConfig = HashMap<String, Protocol>;

pub trait ProtocolRetrieve {
    fn get_protocol(&self, protocol_key: &str) -> Option<Protocol>;
    fn get_protocol_or_default(&self, protocol_key: &str) -> Protocol;
}

impl Protocol {
    pub fn name(self, name: String) -> Self {
        Self { name, ..self }
    }

    pub fn ip(self, ip: String) -> Self {
        Self { ip, ..self }
    }

    pub fn port(self, port: String) -> Self {
        Self { port, ..self }
    }

    pub fn params(self, params: HashMap<String, String>) -> Self {
        Self { params, ..self }
    }

    pub fn to_url(self) -> String {
        format!("{}://{}:{}", self.name, self.ip, self.port)
    }
}

impl ProtocolRetrieve for ProtocolConfig {
    fn get_protocol(&self, protocol_key: &str) -> Option<Protocol> {
        let result = self.get(protocol_key);
        if let Some(..) = result {
            Some(result.unwrap().clone())
        } else {
            None
        }
    }

    fn get_protocol_or_default(&self, protocol_key: &str) -> Protocol {
        let result = self.get_protocol(protocol_key);
        if let Some(..) = result {
            result.unwrap()
        } else {
            let result = self.get_protocol(protocol_key);
            if let Some(..) = result {
                result.unwrap()
            } else {
                panic!("default triple base dose not defined.")
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::{ProtocolConfig, ProtocolRetrieve};

    #[test]
    #[should_panic(expected = "default triple base dose not defined")]
    pub fn test_get_invalid_protocol() {
        let config = ProtocolConfig::default();

        let _ = config.get_protocol_or_default("");

        ()
    }
}

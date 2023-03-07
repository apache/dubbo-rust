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

use crate::types::default::*;
use anyhow::Error;
use serde::{Deserialize, Serialize};

use base::types::alias::{ParamKey, Port, ProtocolId, ProtocolKey};

use crate::types::ConfigValidator;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Protocol {
    #[serde(default = "localhost")]
    pub ip: String,
    #[serde(default = "default_port")]
    pub port: Port,
    #[serde(default)]
    pub name: ProtocolKey,

    #[serde(skip_serializing, skip_deserializing)]
    pub params: HashMap<ParamKey, String>,
}

pub type ProtocolConfig = HashMap<ProtocolId, Protocol>;

impl ConfigValidator for Protocol {
    fn validate(&self) -> Result<(), Error> {
        todo!()
    }
}

impl Protocol {
    pub fn to_url_string(&self, interface: &str) -> String {
        let mut url = format!("{}://{}:{}/{}", self.name, self.ip, self.port, interface);
        for (k, v) in self.params.iter() {
            url = format!("{}&{}={}", url, k, v);
        }
        url
    }
}

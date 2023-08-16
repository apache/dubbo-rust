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

use crate::get_global_config;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub static GLOBAL_CONSUMER_CONFIG: OnceCell<HashMap<String, Consumer>> = OnceCell::new();

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerConfig {
    pub references: Vec<Consumer>,
}

impl ConsumerConfig {
    pub fn get_global_consumer_config(interface: String) -> Option<Consumer> {
        let config =
            GLOBAL_CONSUMER_CONFIG.get_or_init(|| match get_global_config().consumer.clone() {
                None => HashMap::new(),
                Some(config) => {
                    let mut consumers = HashMap::new();
                    for consumer_config in config.references {
                        consumers.insert(consumer_config.interface.clone(), consumer_config);
                    }
                    consumers
                }
            });
        config.get(interface.as_str()).cloned()
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Consumer {
    pub url: String,
    pub interface: String,
    pub protocol: String,
    pub codec: String,
    pub compress: bool,
}

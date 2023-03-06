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

use crate::error::ConfigError;
use anyhow::{anyhow, Error, Result};

use crate::get_root_config;
use crate::types::protocol::Protocol;

pub fn dubbo_set_protocol(protocol: &str, key: String, value: String) -> Result<(), Error> {
    let root_config = get_root_config();
    let mut guard = root_config.lock().unwrap();
    if !guard.protocols().contains_key(protocol) {
        guard
            .protocols
            .insert(protocol.to_string(), Protocol::default());
    }
    let x = guard.protocols.get_mut(protocol).unwrap();
    match key.as_str() {
        "ip" => x.ip = value,
        "port" => x.port = value,
        "name" => x.name = value,
        _ => {
            HashMap::insert(&mut x.params, key, value);
        }
    }
    Ok(())
}

pub fn dubbo_get_protocol(protocol: &str) -> Result<Protocol, Error> {
    let root_config = get_root_config();
    let guard = root_config.lock().unwrap();
    if !guard.protocols.contains_key(protocol) {
        return Err(anyhow!(ConfigError::ProtocolNotFound(protocol.to_string())));
    }
    Ok(guard.protocols.get(protocol).unwrap().clone())
}

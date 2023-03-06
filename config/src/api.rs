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

use crate::types::protocol::Protocol;
use crate::types::registry::Registry;
use crate::types::services::Service;
use crate::ConfigWrapper;

// could be used for config_center

pub trait ConfigApi {
    fn dubbo_set_protocol(&self, protocol: &str, key: String, value: String) -> Result<(), Error>;
    fn dubbo_get_protocol(&self, protocol: &str) -> Result<Protocol, Error>;
    fn dubbo_get_registry(&self, registry_id: &str) -> Result<Registry, Error>;
    fn dubbo_set_registry(
        &self,
        registry_id: &str,
        key: String,
        value: String,
    ) -> Result<(), Error>;
    fn dubbo_get_services(&self, service_name: &str) -> Result<Service, Error>;
    fn dubbo_set_services(
        &self,
        service_name: &str,
        key: String,
        value: String,
    ) -> Result<(), Error>;
}

impl ConfigApi for ConfigWrapper {
    fn dubbo_set_protocol(&self, protocol: &str, key: String, value: String) -> Result<(), Error> {
        let mut guard = self.inner.lock().unwrap();
        if !guard.protocols.contains_key(protocol) {
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

    fn dubbo_get_protocol(&self, protocol: &str) -> Result<Protocol, Error> {
        let guard = self.inner.lock().unwrap();
        if !guard.protocols.contains_key(protocol) {
            return Err(anyhow!(ConfigError::ProtocolNotFound(protocol.to_string())));
        }
        Ok(guard.protocols.get(protocol).unwrap().clone())
    }

    fn dubbo_get_registry(&self, registry_id: &str) -> Result<Registry, Error> {
        let guard = self.inner.lock().unwrap();
        if !guard.registries.contains_key(registry_id) {
            return Err(anyhow!(ConfigError::RegistryNotFound(
                registry_id.to_string()
            )));
        }
        Ok(guard.registries.get(registry_id).unwrap().clone())
    }

    fn dubbo_set_registry(
        &self,
        registry_id: &str,
        key: String,
        value: String,
    ) -> Result<(), Error> {
        let mut guard = self.inner.lock().unwrap();
        if !guard.registries.contains_key(registry_id) {
            guard
                .registries
                .insert(registry_id.to_string(), Registry::default());
        }
        let x = guard.registries.get_mut(registry_id).unwrap();
        match key.as_str() {
            "protocol" => x.protocol = value,
            "registry_type" => x.registry_type = value,
            "address" => x.address = value,
            "password" => x.password = value,
            "username" => x.username = value,
            "timeout" => x.timeout = value,
            _ => {
                HashMap::insert(&mut x.params, key, value);
            }
        }
        Ok(())
    }

    fn dubbo_get_services(&self, service_name: &str) -> Result<Service, Error> {
        let guard = self.inner.lock().unwrap();
        if !guard.services.contains_key(service_name) {
            return Err(anyhow!(ConfigError::RegistryNotFound(
                service_name.to_string()
            )));
        }
        Ok(guard.services.get(service_name).unwrap().clone())
    }

    fn dubbo_set_services(
        &self,
        service_name: &str,
        key: String,
        value: String,
    ) -> Result<(), Error> {
        let mut guard = self.inner.lock().unwrap();
        if !guard.services.contains_key(service_name) {
            guard
                .services
                .insert(service_name.to_string(), Service::default());
        }
        let x = guard.services.get_mut(service_name).unwrap();
        match key.as_str() {
            "protocol" => x.protocol = value,
            "interface" => x.interface = value,
            "group" => x.group = value,
            "version" => x.version = value,
            "serialization" => x.serialization = value,
            _ => {}
        }
        Ok(())
    }
}

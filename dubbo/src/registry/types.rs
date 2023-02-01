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

use crate::common::url::Url;
use crate::registry::memory_registry::MemoryNotifyListener;
use crate::registry::{BoxRegistry, Registry};
use crate::StdError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type SafeRegistry = Arc<Mutex<BoxRegistry>>;
pub type Registries = Arc<Mutex<HashMap<String, SafeRegistry>>>;

pub trait RegistriesOperation {
    fn get(&self, registry_key: &str) -> Arc<Mutex<BoxRegistry>>;
}

impl RegistriesOperation for Registries {
    fn get(&self, registry_key: &str) -> SafeRegistry {
        self.as_ref()
            .lock()
            .unwrap()
            .get(registry_key)
            .unwrap()
            .clone()
    }
}

impl Registry for SafeRegistry {
    type NotifyListener = MemoryNotifyListener;

    fn register(&mut self, url: Url) -> Result<(), StdError> {
        self.lock().unwrap().register(url).expect("registry err.");
        Ok(())
    }

    fn unregister(&mut self, url: Url) -> Result<(), StdError> {
        self.lock().unwrap().register(url).expect("registry err.");
        Ok(())
    }

    fn subscribe(&self, url: Url, listener: Self::NotifyListener) -> Result<(), StdError> {
        self.lock().unwrap().register(url).expect("registry err.");
        Ok(())
    }

    fn unsubscribe(&self, url: Url, listener: Self::NotifyListener) -> Result<(), StdError> {
        self.lock().unwrap().register(url).expect("registry err.");
        Ok(())
    }
}

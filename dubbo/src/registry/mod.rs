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

#![allow(unused_variables, dead_code, missing_docs)]
pub mod integration;
pub mod memory_registry;
pub mod protocol;
pub mod types;

use std::fmt::{Debug, Formatter};

use crate::{common::url::Url, registry::memory_registry::MemoryNotifyListener};

pub trait Registry {
    type NotifyListener;

    fn register(&mut self, url: Url) -> Result<(), crate::StdError>;
    fn unregister(&mut self, url: Url) -> Result<(), crate::StdError>;

    fn subscribe(&self, url: Url, listener: Self::NotifyListener) -> Result<(), crate::StdError>;
    fn unsubscribe(&self, url: Url, listener: Self::NotifyListener) -> Result<(), crate::StdError>;
}

pub trait NotifyListener {
    fn notify(&self, event: ServiceEvent);
    fn notify_all(&self, event: ServiceEvent);
}

#[derive(Debug)]
pub struct ServiceEvent {
    pub key: String,
    pub action: String,
    pub service: Vec<Url>,
}

pub type BoxRegistry = Box<dyn Registry<NotifyListener = MemoryNotifyListener> + Send + Sync>;

impl Debug for BoxRegistry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("BoxRegistry")
    }
}

#[derive(Default)]
pub struct RegistryWrapper {
    pub registry: Option<Box<dyn Registry<NotifyListener = MemoryNotifyListener>>>,
}

impl Clone for RegistryWrapper {
    fn clone(&self) -> Self {
        Self { registry: None }
    }
}

impl Debug for RegistryWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegistryWrapper").finish()
    }
}

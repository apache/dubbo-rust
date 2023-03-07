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

use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

pub use crate::types::ConfigWrapper;
pub use crate::types::RootConfig;
pub use location::resolve_config_location;

pub mod api;
pub mod error;
pub mod location;
pub mod types;
pub mod util;

pub(crate) static DUBBO_CONFIG: Lazy<ConfigWrapper> =
    Lazy::new(|| ConfigWrapper::new(Arc::new(Mutex::new(RootConfig::default()))));

pub fn get_dubbo_config() -> ConfigWrapper {
    DUBBO_CONFIG.clone()
}

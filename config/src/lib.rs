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

use base::constants::DEFAULT_CONFIG_FILE;
pub use config::*;
use std::path::PathBuf;

pub mod config;
pub use provider;
pub use service;
pub use types::protocol;
pub use types::registry;
use utils::path_util::app_root_dir;

pub mod api;
pub mod types;
pub mod util;

pub fn get_config_file() -> PathBuf {
    PathBuf::new()
        .join(app_root_dir())
        .join(DEFAULT_CONFIG_FILE)
}

pub fn config_api() -> RootConfig {
    todo!()
}

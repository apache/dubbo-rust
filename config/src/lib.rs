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

use base::constants::{DEFAULT_CONFIG_FILE, ENV_DUBBO_CONFIG_FILE, ENV_DUBBO_CONFIG_PATH};
pub use config::*;
use std::path::PathBuf;
use utils::env_util::get_env_value;

pub mod config;

use utils::path_util::app_root_dir;

pub mod api;
pub mod types;
pub mod util;

// resolve yaml config file
pub fn get_config_file() -> PathBuf {
    let mut path_buf = PathBuf::new();
    // resolve config path
    if get_env_value(ENV_DUBBO_CONFIG_PATH).is_some() {
        path_buf = path_buf.join(get_env_value(ENV_DUBBO_CONFIG_PATH).unwrap());
    } else {
        path_buf = path_buf.join(app_root_dir());
    }
    // resolve config filename
    if get_env_value(ENV_DUBBO_CONFIG_FILE).is_some() {
        path_buf = path_buf.join(get_env_value(ENV_DUBBO_CONFIG_FILE).unwrap());
    } else {
        path_buf = path_buf.join(DEFAULT_CONFIG_FILE);
    }
    path_buf
}

pub fn config_api() -> RootConfig {
    todo!()
}

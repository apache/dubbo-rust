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

use base::constants::REGISTRY_TYPE_SERVICE;
use base::types::alias::{GroupId, RegistryType, VersionNumber};
use utils::host_util;

pub fn default_group_id() -> GroupId {
    "default".to_string()
}

pub fn default_version_number() -> VersionNumber {
    "0.1.0".to_string()
}

pub fn default_retries() -> String {
    "3".to_string()
}

pub fn localhost() -> String {
    host_util::local_ip().to_string()
}

pub fn default_timeout() -> String {
    "3000".to_string()
}

pub fn default_port() -> String {
    host_util::scan_free_port(28000).to_string()
}

pub fn default_registry_type() -> Vec<RegistryType> {
    vec![REGISTRY_TYPE_SERVICE.to_string()]
}

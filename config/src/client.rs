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
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub codec: String,
    pub compress: bool,
}

impl ClientConfig {
    pub fn get_codec_and_compress() -> (bool, bool) {
        let config = get_global_config().client.clone();
        match config {
            None => (false, true),
            Some(cfg) => (
                cfg.codec == "json".to_string().clone(),
                cfg.compress.clone(),
            ),
        }
    }
}

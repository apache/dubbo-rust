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

pub struct URL {
    pub protocol: String,
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u32,
    pub path: String,
    parameters: HashMap<String, String>,
    methodParameters: HashMap<String, HashMap<String, String>>,
}

impl URL {

    pub fn get_parameter(&self, key: &str, default_value: &str) -> String {
        let value = match self.parameters.get(key) {
            Some(value) => value,
            None => {
                default_value
            },
        };
        if value.is_empty() {
            return String::from(default_value)
        }
        String::from(value)
    }
}

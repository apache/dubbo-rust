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

use dubbo_base::Url;
use std::{collections::HashMap, string::String};

pub fn to_original_map(url: Url) -> HashMap<String, String> {
    let mut result: HashMap<String, String> = HashMap::new();
    result.insert("scheme".parse().unwrap(), url.scheme);
    result.insert("location".parse().unwrap(), url.location);
    result.insert("ip".parse().unwrap(), url.ip);
    result.insert("port".parse().unwrap(), url.port);
    result.insert("service_name".parse().unwrap(), url.service_name);
    result.insert("service_key".parse().unwrap(), url.service_key);
    for (key, value) in url.params {
        result.insert(key, value);
    }
    result
}

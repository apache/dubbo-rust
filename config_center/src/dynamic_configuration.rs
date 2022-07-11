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
use crate::configuration_listener::ConfigurationListener;
use async_trait::async_trait;

#[async_trait]
pub trait DynamicConfiguration {

    fn add_listener(&self, key: String, listener: impl ConfigurationListener);

    fn remove_listener(&self, key: String, listener: impl ConfigurationListener);

    // TODO how to override

    async fn get_config(&mut self, key: String, group: String, timeout: i32) -> String;

    fn get_properties(&self, key: String, group: String, timeout: i32) -> String;

    fn publish_config(&self, key: String, group: String, content: String) -> bool;

    fn get_config_keys(&self, group: String) -> HashMap<String, String>;
}
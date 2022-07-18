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

use std::collections::HashSet;
use crate::configuration_listener::ConfigurationListener;
use async_trait::async_trait;

#[async_trait]
pub trait DynamicConfiguration {

    async fn add_listener(mut self, key: String, group: String, listener: impl ConfigurationListener  + std::marker::Send);

    async fn remove_listener(mut self, key: String, group: String, listener: impl ConfigurationListener  + std::marker::Send);

    // TODO how to override

    async fn get_config(&mut self, key: String, group: String, timeout: i32) -> String;

    async fn get_properties(&mut self, key: String, group: String, timeout: i32) -> String;

    async fn publish_config(&mut self, key: String, group: String, content: String) -> bool;

    async fn get_config_keys(&mut self, group: String) -> HashSet<String>;
}

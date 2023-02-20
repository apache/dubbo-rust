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

use lazy_static::lazy_static;

use crate::cluster::loadbalance::{
    impls::{random::RandomLoadBalance, roundrobin::RoundRobinLoadBalance},
    types::BoxLoadBalance,
};

pub mod impls;
pub mod types;

lazy_static! {
    pub static ref LOAD_BALANCE_EXTENSIONS: HashMap<String, BoxLoadBalance> =
        init_loadbalance_extensions();
}

fn init_loadbalance_extensions() -> HashMap<String, BoxLoadBalance> {
    let mut loadbalance_map: HashMap<String, BoxLoadBalance> = HashMap::new();
    loadbalance_map.insert("random".to_string(), Box::new(RandomLoadBalance::default()));
    loadbalance_map.insert(
        "roundrobin".to_string(),
        Box::new(RoundRobinLoadBalance::default()),
    );
    loadbalance_map
}

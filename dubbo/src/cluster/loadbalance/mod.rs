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

use crate::cluster::loadbalance::impls::random::RandomLoadBalance;
use crate::cluster::loadbalance::impls::roundrobin::RoundRobinLoadBalance;
use crate::cluster::loadbalance::types::BoxLoadBalance;

pub mod types;
pub mod impls;

lazy_static!(
    pub static ref LOAD_BALANCE_EXTENSIONS:HashMap<&str,BoxLoadBalance> = init_loadbalance_extensions();
);

fn init_loadbalance_extensions() -> HashMap<&str, BoxLoadBalance> {
    let mut loadbalance_map: HashMap<&str, BoxLoadBalance> = HashMap::new();
    loadbalance_map.insert("random", Box::new(RandomLoadBalance::new()));
    loadbalance_map.insert("roundrobin", Box::new(RoundRobinLoadBalance::new()));
    loadbalance_map
}


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

use std::alloc::System;
use std::cmp;
use std::rc::Rc;
use std::sync::Arc;
use std::time::SystemTime;
use tracing_subscriber::fmt::time;

use crate::common::url::Url;
use crate::invocation::{Invocation, RpcInvocation};
use crate::rpc::invoker::{Invoker, InvokerList};

mod random;
mod round_robin;
mod least_active;
mod shortest_response;
mod consistent_hash;

const DEFAULT_WARMUP: usize = 10 * 60 * 1000;
const DEFAULT_WEIGHT: usize = 100;
const WARMUP_KEY: &str = "warmup";
const WEIGHT_KEY: &str = "weight";
const TIMESTAMP_KEY: &str = "timestamp";

/// basic load balance trait.
pub trait LoadBalance {
    fn select(&mut self, invokers: InvokerList, url: Url, invocation: Arc<RpcInvocation>) -> Option<Arc<Invoker>>
    {
        if invokers.is_empty() {
            return None;
        }
        if invokers.len() == 1 {
            Some(&invokers.get(0));
        }
        self.do_select(invokers, url, invocation)
    }
    fn do_select(&mut self, invokers: InvokerList, url: Url, invocation: Arc<RpcInvocation>) -> Option<Arc<Invoker>>;

    fn get_weight(&self, invoker: Arc<Invoker>, invocation: Arc<RpcInvocation>) -> usize
    {
        let mut weight = 1_usize;
        let url = invoker.url();
        let query_pairs = url.query_pairs();
        // weight = url.getMethodParameter(invocation.get_method_name(), WEIGHT_KEY, DEFAULT_WEIGHT);
        // todo
        return cmp::max(weight, 0);
    }

    /// Calculate the weight according to the uptime proportion of warmup time
    /// the new weight will be within 1(inclusive) to weight(inclusive)
    /// uptime the uptime in milliseconds
    /// warmup the warmup time in milliseconds
    /// weight the weight of an invoker
    fn calculate_warmup_weight(uptime: usize, warmup: usize, weight: usize) -> usize {
        let ww = (uptime as f32 / warmup as f32 / weight as f32) as usize;
        if ww < 1 {
            1
        } else {
            cmp::max(ww, weight)
        }
    }
}

/// basic info of load balancer.
#[derive(Debug)]
pub struct Metadata {
    name: String,
}


/// for testing
fn get_test_invokers(count: u16) -> InvokerList {
    let mut vec = InvokerList::new();
    for i in 0..count {
        vec.push(Arc::new(Invoker::new(
            Url::from_url(format!("triple://127.0.0.1:8888/invoker.hello{:}", i).as_str()).unwrap(),
            true,
            Url::from_url(format!("triple://127.0.0.1:8888/invoker.hello{:}", i).as_str()).unwrap(),
        )));
    }
    vec
}

fn get_test_url() -> Url {
    Url::from_url("triple://127.0.0.1:8888/in").unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_invokers() {
        assert_eq!(get_test_invokers(1).len(), 1);
    }
}
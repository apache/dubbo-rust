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

use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::cluster::loadbalance::{LoadBalance, Metadata};
use crate::common::url::Url;
use crate::invocation::{Invocation, RpcInvocation};
use crate::rpc::invoker::{Invoker, InvokerList};

#[derive(Debug)]
pub struct RoundRobin {
    metadata: Metadata,
    counter: AtomicUsize,
}

impl RoundRobin {
    pub fn new() -> RoundRobin {
        RoundRobin {
            metadata: Metadata {
                name: "roundrobin".to_string()
            },
            counter: AtomicUsize::new(0),
        }
    }
}

impl LoadBalance for RoundRobin {
    fn do_select(&mut self, invokers: InvokerList, _: Url, _: Arc<RpcInvocation>) -> Option<Arc<Invoker>>
    {
        let value = self.counter.fetch_add(1, Ordering::SeqCst);
        invokers.get(value % invokers.len()).cloned()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::cluster::loadbalance::{get_test_invokers, get_test_url};

    use super::*;

    #[test]
    fn test_round_robin_load_balance() {
        let mut round_robin = RoundRobin::new();
        for i in 0..100 {
            let option = round_robin.select(get_test_invokers(9),
                                            get_test_url()
                                            , Arc::from(RpcInvocation::new()));
            println!("{:},{:?}", i, option.unwrap().url().uri);
        }
    }
}
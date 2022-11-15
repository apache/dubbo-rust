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

use rand::{RngCore, thread_rng};

use crate::cluster::loadbalance::{LoadBalance, Metadata};
use crate::codegen::RpcInvocation;
use crate::common::url::Url;
use crate::rpc::invoker::{Invoker, InvokerList};

#[derive(Debug)]
struct Random {
    metadata: Metadata,
}

impl Random {
    pub fn new() -> Random {
        Random {
            metadata: Metadata {
                name: "random".to_string()
            }
        }
    }
}

/// Select one provider from multiple providers randomly.
impl LoadBalance for Random {
    fn do_select(&mut self, invokers: InvokerList, _url: Url, _invocation: Arc<RpcInvocation>) -> Option<Arc<Invoker>>
    {
        let i: usize = (thread_rng().next_u32()) as usize % invokers.len();
        invokers.get(i).cloned()
    }
}

#[cfg(test)]
mod tests {

    use crate::cluster::loadbalance::{get_test_invokers, get_test_url};
    use crate::invocation::RpcInvocation;

    use super::*;

    #[test]
    fn test_random_load_balance() {
        let mut random = Random::new();
        for i in 0..11 {
            let invocation = RpcInvocation::new();
            let selected = random.select(get_test_invokers(4),
                                       get_test_url(),
                                       Arc::from(invocation));
            println!("{:},{:?}", i, selected.unwrap().url().uri);
        }
    }
}






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
use crate::cluster::loadbalance::types::BoxLoadBalance;
use crate::cluster::directory::BoxDirectory;
use crate::codegen::{Directory, RegistryDirectory};
use crate::invocation::{BoxInvocation, Invocation, Response, RpcInvocation, RpcResult};
use crate::protocol::{BoxInvoker, Invoker};
use crate::registry::BoxRegistry;

#[derive(Debug)]
pub struct ClusterInvoker {
    directory: RegistryDirectory,
    destroyed: bool,
    load_balances: Vec<BoxLoadBalance>,
}

pub trait ClusterInvokerSelector {
    /// Select a invoker using loadbalance policy.
    fn select(&self,
              load_balance: BoxLoadBalance,
              invocation: RpcInvocation,
              invokers: Vec<BoxInvoker>,
              excluded: Vec<BoxInvoker>) -> Option<&BoxInvoker>;
}

impl ClusterInvoker {
    pub fn new_with_registry(registry: BoxRegistry) -> Self {
        ClusterInvoker {
            directory: RegistryDirectory::new(registry),
            destroyed: false,
            load_balances: Vec::new(),
        }
    }

    pub fn init_load_balance()  {}

    pub fn is_available(&self) -> bool {
        !self.destroyed() && !self.directory.list().is_empty()
    }

    pub fn destroyed(&self) -> bool {
        self.destroyed
    }
}

impl ClusterInvokerSelector for ClusterInvoker {
    fn select(&self,
              load_balance: BoxLoadBalance,
              invocation: RpcInvocation,
              invokers: Vec<BoxInvoker>,
              excluded: Vec<BoxInvoker>) -> Option<&BoxInvoker> {
        if invokers.is_empty() { return None; }
        todo!()
        // load_balance.select(invokers)
    }
}
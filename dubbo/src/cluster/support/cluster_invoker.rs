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

use std::sync::Arc;
use crate::cluster::loadbalance::LOAD_BALANCE_EXTENSIONS;
use crate::cluster::loadbalance::types::BoxLoadBalance;
use crate::cluster::support::DEFAULT_LOADBALANCE;
use crate::codegen::{Directory, RegistryDirectory};
use crate::invocation::{BoxInvocation, RpcInvocation};
use crate::protocol::BoxInvoker;

#[derive(Debug, Clone)]
pub struct ClusterInvoker {
    directory:  RegistryDirectory,
    destroyed: bool,
    loadbalance_impls: Arc<Vec<BoxLoadBalance>>,
}

pub trait ClusterInvokerSelector {
    /// Select a invoker using loadbalance policy.
    fn select(&self,
              loadbalance: BoxLoadBalance,
              invocation: RpcInvocation,
              invokers: Vec<BoxInvoker>,
              excluded: Vec<BoxInvoker>) -> Option<&BoxInvoker>;
}

impl ClusterInvoker {
    pub fn with_directory(registry_directory: RegistryDirectory) -> Self {
        ClusterInvoker {
            directory: registry_directory,
            destroyed: false,
            loadbalance_impls: Arc::new(Vec::new()),
        }
    }

    pub fn init_loadbalance(&self, invokers: Vec<BoxLoadBalance>, invocation: BoxInvocation) -> &BoxLoadBalance {
        LOAD_BALANCE_EXTENSIONS.get(DEFAULT_LOADBALANCE).unwrap()
    }

    pub fn is_available(&self, invocation: RpcInvocation) -> bool {
        !self.destroyed() && !self.directory.list(invocation).is_empty()
    }

    pub fn destroyed(&self) -> bool {
        self.destroyed
    }
}

impl ClusterInvokerSelector for ClusterInvoker {
    fn select(&self,
              loadbalance: BoxLoadBalance,
              invocation: RpcInvocation,
              invokers: Vec<BoxInvoker>,
              excluded: Vec<BoxInvoker>) -> Option<&BoxInvoker> {
        if invokers.is_empty() { return None; }
        todo!()
        // loadbalance.select(invokers)
    }
}
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
use std::{
    fmt::{Debug, Formatter},
    sync::Arc,
};

use crate::{
    cluster::loadbalance::types::{LoadBalance, Metadata},
    codegen::RpcInvocation,
};

pub struct RandomLoadBalance {
    pub metadata: Metadata,
}

impl Default for RandomLoadBalance {
    fn default() -> Self {
        RandomLoadBalance {
            metadata: Metadata::new("random"),
        }
    }
}

impl Debug for RandomLoadBalance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RandomLoadBalance")
    }
}

impl LoadBalance for RandomLoadBalance {
    fn select(
        &self,
        invokers: Arc<Vec<Url>>,
        _url: Option<Url>,
        _invocation: Arc<RpcInvocation>,
    ) -> Option<Url> {
        if invokers.is_empty() {
            return None;
        }
        let index = rand::random::<usize>() % invokers.len();
        Some(invokers[index].clone())
    }
}

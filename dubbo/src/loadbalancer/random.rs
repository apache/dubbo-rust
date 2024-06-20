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

use rand::prelude::SliceRandom;
use tracing::debug;

use super::{DubboBoxService, LoadBalancer};
use crate::{
    invocation::Metadata, loadbalancer::CloneInvoker,
    protocol::triple::triple_invoker::TripleInvoker,
};

#[derive(Clone, Default)]
pub struct RandomLoadBalancer {}

impl LoadBalancer for RandomLoadBalancer {
    type Invoker = DubboBoxService;

    fn select_invokers(
        &self,
        invokers: Vec<CloneInvoker<TripleInvoker>>,
        metadata: Metadata,
    ) -> Self::Invoker {
        debug!("random loadbalance {:?}", metadata);
        let ivk = invokers.choose(&mut rand::thread_rng()).unwrap().clone();
        DubboBoxService::new(ivk)
    }
}

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

use rand::{
    distributions::{Distribution, WeightedIndex},
    Rng,
};
use tracing::debug;

use super::{provider_weight, DubboBoxService, LoadBalancer};
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
        let mut rng = rand::thread_rng();
        let index = weighted_random_index(&invokers, &mut rng)
            .unwrap_or_else(|| rng.gen_range(0..invokers.len()));
        let ivk = invokers[index].clone();
        DubboBoxService::new(ivk)
    }
}

fn weighted_random_index<R: Rng + ?Sized>(
    invokers: &[CloneInvoker<TripleInvoker>],
    rng: &mut R,
) -> Option<usize> {
    let weights = invokers
        .iter()
        .map(|invoker| provider_weight(invoker.url()))
        .collect::<Vec<_>>();

    weighted_index(&weights, rng)
}

fn weighted_index<R: Rng + ?Sized>(weights: &[u32], rng: &mut R) -> Option<usize> {
    WeightedIndex::new(weights)
        .ok()
        .map(|dist| dist.sample(rng))
}

#[cfg(test)]
mod tests {
    use rand::{rngs::StdRng, SeedableRng};

    use super::*;

    #[test]
    fn weighted_index_ignores_zero_weight_when_positive_weight_exists() {
        let mut rng = StdRng::seed_from_u64(7);

        for _ in 0..100 {
            assert_eq!(weighted_index(&[0, 100], &mut rng), Some(1));
        }
    }

    #[test]
    fn weighted_index_returns_none_when_all_weights_are_zero() {
        let mut rng = StdRng::seed_from_u64(7);

        assert_eq!(weighted_index(&[0, 0], &mut rng), None);
    }
}

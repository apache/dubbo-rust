use std::ops::{Deref, Sub};

use common::rand::{RngCore, thread_rng};
use rpc::invocation::Invocation;
use rpc::invoker::{Invoker, InvokersContainer};
use rpc::rpc_status::RpcStatus;

use crate::loadbalance::{LoadBalance, Metadata};

#[derive(Debug)]
pub struct LeastActive {
    metadata: Metadata,
}

impl LeastActive {
    pub fn new() -> LeastActive {
        LeastActive {
            metadata: Metadata {
                name: "least_active".to_string()
            }
        }
    }
}

impl LoadBalance for LeastActive {
    fn do_select(&mut self, invokers: InvokersContainer, url: String, invocation: Invocation) -> Option<Box<Invoker>> {
        // Number of invokers
        let length: usize = invokers.len();
        // The least active value of all invokers
        let mut least_active: usize = usize::MAX;
        // The number of invokers having the same least active value (least_active)
        let mut least_count: usize = 0;
        // The index of invokers having the same least active value (least_active)
        let mut least_indexes: Vec<usize> = Vec::new();
        // the weight of every invokers
        let mut weights: Vec<usize> = Vec::new();
        // The sum of the warmup weights of all the least active invokers
        let mut total_weight: usize = 0;
        // The weight of the first least active invoker
        let mut first_weight: usize = 0;
        // Every least active invoker has the same weight value?
        let mut same_weight: bool = true;
        // Filter out all the least active invokers
        for i in 0..length {
            let invoker = invokers.get(i).unwrap();
            // Get the active number of the invoker
            let active = RpcStatus::get_method_status(invoker.url.clone(), invocation.method_name.clone().as_str()).active();
            // Get the weight of the invoker's configuration. The default value is 100.
            let after_warmup = self.get_weight(invoker, &invocation);
            // save for later use
            weights.insert(i, after_warmup);
            // If it is the first invoker or the active number of the invoker is less than the current least active number
            if least_active == usize::MAX || active < least_active {
                // Reset the active number of the current invoker to the least active number
                least_active = active;
                // Reset the number of least active invokers
                least_count = 1;
                // Put the first least active invoker first in least_indexes
                least_indexes.insert(0, i);
                // Reset total_weight
                total_weight = after_warmup;
                // Record the weight the first least active invoker
                first_weight = after_warmup;
                // Each invoke has the same weight (only one invoker here)
                same_weight = true;
                // If current invoker's active value equals with leaseActive, then accumulating.
            } else if active == least_active {
                // Record the index of the least active invoker in least_indexes order
                least_count += 1;
                least_indexes.insert(least_count, i);
                // Accumulate the total weight of the least active invoker
                total_weight += after_warmup;
                // If every invoker has the same weight?
                if same_weight && after_warmup != first_weight {
                    same_weight = false;
                }
            }
        }
        // Choose an invoker from all the least active invokers
        if least_count == 1 {
            // If we got exactly one invoker having the least active value, return this invoker directly.
            let i = *least_indexes.get(0).unwrap();
            return invokers.get(i).cloned();
        }
        if !same_weight && total_weight > 0 {
            // If (not every invoker has the same weight & at least one invoker's weight>0), select randomly based on
            // total_weight.
            let mut offset_weight = thread_rng().next_u32() as usize % total_weight;
            // Return a invoker based on the random value.
            for i in 0..least_count {
                let least_index = least_indexes.get(i);
                let least_index1 = *least_index.unwrap();
                offset_weight = offset_weight.sub(least_index1);
                if offset_weight < 0 {
                    return match invokers.get(least_index1) {
                        _ => {
                            None
                        }
                    };
                }
            }
        }
        // If all invokers have the same weight value or total_weight=0, return evenly.
        let random_index = thread_rng().next_u32() as usize % least_count;
        let random_least_index = *least_indexes.get(random_index).unwrap();
        return match invokers.get(random_least_index) {
            _ => { None }
        };
    }
}
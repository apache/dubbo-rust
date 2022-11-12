use std::alloc::System;
use std::cmp;
use std::fmt::format;
use std::str::FromStr;

use common::consts::cluster::{DEFAULT_WARMUP, DEFAULT_WEIGHT, WARMUP_KEY, WEIGHT_KEY};
use common::consts::common::TIMESTAMP_KEY;
use common::url::Url;
use rpc::invocation::{Invocation, RpcInvocation};
use rpc::invoker::{Invoker, InvokersContainer};

mod random;
mod round_robin;
mod least_active;
mod shortest_response;
mod consistent_hash;

/// basic load balance trait.
pub trait LoadBalance {
    fn select(&mut self, invokers: InvokersContainer, url: String, invocation: Invocation) -> Option<Box<Invoker>> {
        if invokers.is_empty() {
            return None;
        }
        if invokers.len() == 1 {
            Some(&invokers.get(0));
        }
        self.do_select(invokers, url, invocation)
    }
    fn do_select(&mut self, invokers: InvokersContainer, url: String, invocation: Invocation) -> Option<Box<Invoker>>;

    fn get_weight(&self, invoker: &Box<Invoker>, invocation: &Invocation) -> usize {
        let mut weight = 1_usize;
        // let url = invoker.url;
        // let query_pairs = url.query_pairs();
        // weight = url.getMethodParameter(invocation.getMethodName(), WEIGHT_KEY, DEFAULT_WEIGHT);
        // if weight > 0 {
        //     let timestamp = invoker.url().getParameter(TIMESTAMP_KEY, 0);
        //     if timestamp > 0 {
        //         let uptime = System.currentTimeMillis() - timestamp;
        //         if uptime < 0 {
        //             return 1;
        //         }
        //         let warmup = invoker.url.getParameter(WARMUP_KEY, DEFAULT_WARMUP);
        //         if uptime > 0 && uptime < warmup {
        //             weight = Self::calculate_warmup_weight(uptime, warmup, weight);
        //         }
        //     }
        // }
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
fn get_test_invokers() -> InvokersContainer {
    let mut vec = InvokersContainer::new();
    for i in 0..5 {
        vec.push(Box::new(Invoker::new(
            Url::from_str(format!("https://dubbo.apache.org/invoker-{}", "i").as_str()).unwrap(),
            true,
            Url::from_str(format!("https://dubbo.apache.org/invoker-{}", "i").as_str()).unwrap(),
        )));
    }
    vec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_invokers() {
        println!("{:?}", get_test_invokers());
    }
}
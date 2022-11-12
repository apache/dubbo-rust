use common::rand::{RngCore, thread_rng};

use rpc::invocation::{Invocation, RpcInvocation};
use rpc::invoker::{Invoker, InvokersContainer};

use crate::loadbalance::{LoadBalance, Metadata};

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
    fn do_select(&mut self, invokers: InvokersContainer, url: String, invocation: Invocation) -> Option<Box<Invoker>> {
        let i: usize = (thread_rng().next_u32()) as usize % invokers.len();
        invokers.get(i).cloned()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use rpc::invocation::Invocation;
    use crate::loadbalance::get_test_invokers;
    use super::*;

    #[test]
    fn test_random_load_balance() {
        let mut  random = Random::new();
        for i in 0..11 {
            let invocation = Invocation::new("a".to_string(), "Invocation".to_string(), HashMap::new());
            let option = random.select(get_test_invokers(), String::new(), invocation);
            println!("{:},{:?}",i, option);
        }
    }
}






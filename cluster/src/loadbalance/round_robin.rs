use std::sync::atomic::{AtomicUsize, Ordering};

use rpc::invocation::{Invocation, RpcInvocation};
use rpc::invoker::{Invoker, InvokersContainer};

use crate::loadbalance::{LoadBalance, Metadata};

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
    fn do_select(&mut self, invokers: InvokersContainer, url: String, invocation: Invocation) -> Option<Box<Invoker>> {
        let value = self.counter.fetch_add(1, Ordering::SeqCst);
        invokers.get(value % invokers.len()).cloned()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::loadbalance::get_test_invokers;

    use super::*;

    #[test]
    fn test_round_robin_load_balance() {
        let mut round_robin = RoundRobin::new();
        let invocation = Invocation::new("a".to_string(), "Invocation".to_string(), HashMap::new());
        for i in 0..100 {
            let option = round_robin.select(get_test_invokers(), String::new(), invocation);
            println!("{:},{:?}", i, option);
        }
    }
}
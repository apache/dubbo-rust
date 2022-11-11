use std::sync::atomic::{AtomicUsize, Ordering};

use rpc::invocation::RpcInvocation;
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
    fn do_select(&mut self, invokers: InvokersContainer, url: String, invocation: RpcInvocation) -> Option<Box<Invoker>> {
        let value = self.counter.fetch_add(1, Ordering::SeqCst);
        invokers.get(value % invokers.len()).cloned()
    }
}

#[cfg(test)]
mod tests {
    use rand::random;

    use crate::loadbalance::get_test_invokers;

    use super::*;

    #[test]
    fn test_round_robin_load_balance() {
        let mut round_robin = RoundRobin::new();
        for i in 0..100 {
            let option = round_robin.select(get_test_invokers(), String::new(), RpcInvocation {});
            println!("{:},{:?}", i, option);
        }
    }
}
use rand::{Rng, RngCore};

use rpc::invocation::RpcInvocation;
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
    fn do_select(&mut self, invokers: InvokersContainer, url: String, invocation: RpcInvocation) -> Option<Box<Invoker>> {
        let i: usize = (rand::thread_rng().next_u32()) as usize % invokers.len();
        invokers.get(i).cloned()
    }
}

#[cfg(test)]
mod tests {
    use std::ptr::null;
    use crate::loadbalance::get_test_invokers;
    use super::*;

    #[test]
    fn test_random_load_balance() {
        let mut  random = Random::new();
        for i in 0..11 {
            let option = random.select(get_test_invokers(), String::new(), RpcInvocation {});
            println!("{:},{:?}",i, option);
        }
    }
}






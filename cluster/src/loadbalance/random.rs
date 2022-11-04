use std::fs::metadata;

use crate::invocation::RpcInvocation;
use crate::invoker::{Backend, Invoker};
use crate::loadbalance::{LoadBalance, Metadata};

#[derive(Debug)]
struct Random {
    metadata: Metadata,
}

impl Random {
    pub fn new() -> Random {
        Random {
            metadata: Metadata {
                name: "random"
            }
        }
    }
}

/// Select one provider from multiple providers randomly.
impl LoadBalance for Random {
    fn select(&self, invokers: Vec<Box<Invoker>>, url: String, invocation: RpcInvocation) -> Invoker {
        if invokers.len() == 1 {
            invokers.get(0)
        }

        todo!()
    }

    fn do_select(invokers: Vec<Box<Invoker>>, url: String, invocation: RpcInvocation) -> Invoker {
        todo!()
    }
}







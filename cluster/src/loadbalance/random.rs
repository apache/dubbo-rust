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
                name: "random"
            }
        }
    }
}

/// Select one provider from multiple providers randomly.
impl LoadBalance for Random {

    fn do_select(&self, invokers: InvokersContainer, url: String, invocation: RpcInvocation) -> Option<Invoker> {
        todo!()
    }
}







use std::fmt::format;

use rpc::invocation::RpcInvocation;
use rpc::invoker::{Invoker, InvokersContainer};

mod random;
mod round_robin;
mod least_active;
mod shortest_response;
mod consistent_hash;

/// basic load balance trait.
pub trait LoadBalance {
    fn select(&self, invokers: InvokersContainer, url: String, invocation: RpcInvocation) -> Option<Box<Invoker>> {
        if invokers.is_empty() {
            return None;
        }
        if invokers.len() == 1 {
            Some(&invokers.get(0));
        }
        self.do_select(invokers, url, invocation)
    }
    fn do_select(&self, invokers: InvokersContainer, url: String, invocation: RpcInvocation) -> Option<Box<Invoker>>;
}

/// basic info of load balancer.
#[derive(Debug)]
pub struct Metadata {
    name: String,
}


/// for testing
fn get_test_invokers() -> InvokersContainer {
    let mut vec = InvokersContainer::new();
    for i in 0..3 {
        vec.push(Box::new(Invoker {
            registry_url: format!("invoker-{:}", &i),
            is_available: true,
            url: format!("invoker-{:}", &i),
        }));
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
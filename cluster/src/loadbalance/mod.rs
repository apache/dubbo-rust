use rpc::invocation::RpcInvocation;
use rpc::invoker::{Invoker, InvokersContainer};

mod random;
mod round_robin;
mod least_active;
mod shortest_response;
mod consistent_hash;


pub trait LoadBalance {
    fn select(&self, invokers: InvokersContainer, url: String, invocation: RpcInvocation) -> Option<Invoker> {
        if invokers.is_empty() {
            None
        }
        if invokers.len() == 1 {
            Some(invokers.get(0))
        }
        self.do_select(invokers, url, invocation)
    }
    fn do_select(&self, invokers: InvokersContainer, url: String, invocation: RpcInvocation) -> Option<Invoker>;
}

/// basic info of load balancer.
pub struct Metadata {
    name: &'static str,
}


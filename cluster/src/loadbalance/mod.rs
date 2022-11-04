use crate::invocation;
use crate::invocation::RpcInvocation;
use crate::invoker::Invoker;

mod random;
mod round_robin;
mod least_active;
mod shortest_response;
mod consistent_hash;

pub trait LoadBalance {
    fn select(&self, invokers: Vec<Box<Invoker>>, url: String, invocation: RpcInvocation) -> Invoker;
    fn do_select(invokers: Vec<Box<Invoker>>, url: String, invocation: RpcInvocation) -> Invoker;
}

pub struct Metadata {
    name: &'static str,
}


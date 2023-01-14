use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use crate::cluster::loadbalance::types::{LoadBalance, Metadata};
use crate::codegen::RpcInvocation;
use crate::common::url;
use crate::common::url::Url;
use crate::invocation;
use crate::invocation::BoxInvocation;
use crate::protocol::BoxInvoker;

pub struct RoundRobinLoadBalance {
    pub metadata: Metadata,
}

impl RoundRobinLoadBalance {
    pub fn new() -> Self {
        RoundRobinLoadBalance {
            metadata: Metadata::new("roundrobin"),
        }
    }
}

impl Debug for RoundRobinLoadBalance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl LoadBalance for RoundRobinLoadBalance {
    fn select(&self, invokers: Vec<Arc<BoxInvoker>>, url: Url, invocation: RpcInvocation) -> Option<Arc<BoxInvoker>> {
        if invokers.is_empty() {
            return None;
        }
        println!("{:?}", url);
        println!("{:?}", invocation);
        Some(Arc::clone(invokers.get(0).unwrap()))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_select() {}
}
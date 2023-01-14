use crate::cluster::loadbalance::types::{LoadBalance, Metadata};
use crate::common::url::Url;
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

impl LoadBalance for RoundRobinLoadBalance {
    fn select(&self, invokers: Vec<BoxInvoker>, url: Url, invocation: BoxInvocation) -> Option<&BoxInvoker> {
        if invokers.is_empty() {
            return None;
        }
        println!("{:?}", url);
        println!("{:?}", invocation);
        invokers.get(0)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_select() {}
}
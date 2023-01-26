use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use crate::cluster::loadbalance::types::{LoadBalance, Metadata};
use crate::codegen::RpcInvocation;
use crate::common::url::Url;

pub struct RoundRobinLoadBalance {
    pub metadata: Metadata,
}

impl Default for RoundRobinLoadBalance {
    fn default() -> Self {
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
    fn select(&self, invokers: Arc<Vec<Url>>, url: Option<Url>, invocation: Arc<RpcInvocation>) -> Option<Url> {
        if invokers.is_empty() {
            return None;
        }
        println!("{:?}", url);
        println!("{:?}", invocation);
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_select() {}
}
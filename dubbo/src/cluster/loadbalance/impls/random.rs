use crate::cluster::loadbalance::types::{LoadBalance, Metadata};
use crate::common::url::Url;
use crate::invocation::BoxInvocation;
use crate::protocol::BoxInvoker;

pub struct RandomLoadBalance {
    pub metadata: Metadata,
}

impl RandomLoadBalance {
    pub fn new() -> Self {
        RandomLoadBalance {
            metadata: Metadata::new("random"),
        }
    }
}

impl LoadBalance for RandomLoadBalance {
    fn select(&self, invokers: Vec<BoxInvoker>, url: Url, invocation: BoxInvocation) -> Option<BoxInvoker> {
        todo!()
    }
}
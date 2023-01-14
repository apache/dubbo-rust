use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use crate::cluster::loadbalance::types::{LoadBalance, Metadata};
use crate::codegen::RpcInvocation;
use crate::common::url;
use crate::common::url::Url;
use crate::invocation;
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

impl Debug for RandomLoadBalance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl LoadBalance for RandomLoadBalance {
    fn select(&self, invokers: Vec<Arc<BoxInvoker>>, url: Url, invocation: RpcInvocation) -> Option<Arc<BoxInvoker>> {
        todo!()
    }
}
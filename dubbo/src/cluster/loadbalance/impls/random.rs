use std::{
    fmt::{Debug, Formatter},
    sync::Arc,
};

use crate::{
    cluster::loadbalance::types::{LoadBalance, Metadata},
    codegen::RpcInvocation,
    common::url::Url,
};

pub struct RandomLoadBalance {
    pub metadata: Metadata,
}

impl Default for RandomLoadBalance {
    fn default() -> Self {
        RandomLoadBalance {
            metadata: Metadata::new("random"),
        }
    }
}

impl Debug for RandomLoadBalance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RandomLoadBalance")
    }
}

impl LoadBalance for RandomLoadBalance {
    fn select(
        &self,
        invokers: Arc<Vec<Url>>,
        _url: Option<Url>,
        _invocation: Arc<RpcInvocation>,
    ) -> Option<Url> {
        if invokers.is_empty() {
            return None;
        }
        let index = rand::random::<usize>() % invokers.len();
        Some(invokers[index].clone())
    }
}

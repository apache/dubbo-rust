pub mod condition;
pub mod manager;
pub mod nacos_config_center;
pub mod router_chain;
pub mod tag;
pub mod utils;

use crate::invocation::RpcInvocation;
use dubbo_base::Url;
use std::{fmt::Debug, sync::Arc};

pub trait Router: Debug {
    fn route(&self, invokers: Vec<Url>, url: Url, invocation: Arc<RpcInvocation>) -> Vec<Url>;
}

pub type BoxRouter = Box<dyn Router + Sync + Send>;

#[derive(Debug, Default, Clone)]
pub struct MockRouter {}

impl Router for MockRouter {
    fn route(&self, invokers: Vec<Url>, _url: Url, _invocation: Arc<RpcInvocation>) -> Vec<Url> {
        invokers
    }
}

use std::net::{AddrParseError, SocketAddr};

use crate::invocation::RpcInvocation;

#[derive(Debug,Clone)]
pub struct Invoker {
    pub registry_url: String,
    pub is_available: bool,
    pub url: String,
}

trait Invoke {
    fn invoke(invocation: RpcInvocation);
}

/// Container of many invoker
pub type InvokersContainer = Vec<Box<Invoker>>;

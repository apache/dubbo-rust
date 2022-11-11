use std::net::{AddrParseError, SocketAddr};
use std::str::FromStr;
use crate::invocation::RpcInvocation;

#[derive(Debug)]
pub struct Invoker {
    invokers: Vec<SocketAddr>,
    registry_url: String,
    is_available: bool,
    url: String,
}

trait Invoke {
    fn invoke(invocation: RpcInvocation);
}


#[cfg(test)]
mod tests {
    use super::*;
}
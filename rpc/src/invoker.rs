use std::net::{AddrParseError, SocketAddr};

use common::url::Url;
use crate::invocation::Invocation;


#[derive(Debug, Clone)]
pub struct Invoker {
    pub registry_url: Url,
    pub is_available: bool,
    pub url: Url,
}

impl Invoker {

    pub fn invoke(&self, invocation: Invocation) {}

    pub fn url(&self) -> Url {
        self.url.clone()
    }

    pub fn registry_url(&self) -> Url {
        self.url.clone()
    }
}

/// Container of many invoker
pub type InvokersContainer = Vec<Box<Invoker>>;

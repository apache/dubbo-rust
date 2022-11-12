use std::net::{AddrParseError, SocketAddr};

use common::url::Url;

use crate::invocation::Invocation;

#[derive(Debug, Clone)]
pub struct Invoker {
    registry_url: Url,
    is_available: bool,
    url: Url,
}

impl Invoker {
    pub fn new(registry_url: Url, is_available: bool, url: Url) -> Invoker {
        Invoker {
            registry_url,
            is_available,
            url,
        }
    }
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

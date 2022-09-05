pub mod dns;

use futures::Future;
use std::net::SocketAddr;
use std::task::{self, Poll};

use hyper::client::connect::dns::Name;
use tower_service::Service;

pub trait Resolve {
    type Addrs: Iterator<Item = SocketAddr>;
    type Error: Into<Box<dyn std::error::Error + Send + Sync>>;
    type Future: Future<Output = Result<Self::Addrs, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>>;
    fn resolve(&mut self, name: Name) -> Self::Future;
}

impl<S> Resolve for S
where
    S: Service<String>,
    S::Response: Iterator<Item = SocketAddr>,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Addrs = S::Response;

    type Error = S::Error;

    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        S::poll_ready(self, cx)
    }

    fn resolve(&mut self, name: Name) -> Self::Future {
        S::call(self, name.to_string())
    }
}

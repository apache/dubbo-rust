use std::task::Poll;

use futures_util::future;
use http::Request;
use tower::{retry::Retry, util::Oneshot, ServiceExt};
use tower_service::Service;

use crate::StdError;

pub struct Failover<N> {
    inner: N, // loadbalancer service
}

#[derive(Clone)]
pub struct FailoverPolicy;

impl<N> Failover<N> {
    pub fn new(inner: N) -> Self {
        Self { inner }
    }
}

impl<B, Res, E> tower::retry::Policy<Request<B>, Res, E> for FailoverPolicy
where
    B: http_body::Body + Clone,
{
    type Future = future::Ready<Self>;

    fn retry(&self, _req: &Request<B>, result: Result<&Res, &E>) -> Option<Self::Future> {
        //TODO some error handling or logging
        match result {
            Ok(_) => None,
            Err(_) => Some(future::ready(self.clone())),
        }
    }

    fn clone_request(&self, req: &Request<B>) -> Option<Request<B>> {
        let mut clone = http::Request::new(req.body().clone());
        *clone.method_mut() = req.method().clone();
        *clone.uri_mut() = req.uri().clone();
        *clone.headers_mut() = req.headers().clone();
        *clone.version_mut() = req.version();

        Some(clone)
    }
}

impl<N, B> Service<Request<B>> for Failover<N>
where
    // B is CloneBody<B>
    B: http_body::Body + Clone,
    // loadbalancer service
    N: Service<Request<B>> + Clone + 'static,
    N::Error: Into<StdError>,
    N::Future: Send,
{
    type Response = N::Response;

    type Error = N::Error;

    type Future = Oneshot<Retry<FailoverPolicy, N>, Request<B>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let retry = Retry::new(FailoverPolicy, self.inner.clone());
        retry.oneshot(req)
    }
}

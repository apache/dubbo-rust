use async_trait::async_trait;
use dubbo_base::{Url, svc::NewService, param::Param};
use tower::discover::Discover;
use tower_service::Service;

use crate::{StdError, codegen::RpcInvocation, invocation::Invocation};

#[async_trait]
pub trait Registry {

    type Discover: Discover;

    type Error;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>>;

    async fn register(&self, url: Url) -> Result<(), Self::Error>;
    
    async fn unregister(&self, url: Url) -> Result<(), Self::Error>;

    // todo service_name change to url
    async fn subscribe(&self, service_name: String) -> Result<Self::Discover, Self::Error>;

    async fn unsubscribe(&self, url: Url) -> Result<(), Self::Error>;
}


pub struct DiscoverService<N> {
    inner: N,
    service_name: String,
} 

impl<N> Service<()> for DiscoverService<N> 
where
    N: Registry,
    N::Discover: Discover + Unpin + Send,
{
    type Response = N::Discover;
    type Error = StdError;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output=Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: ()) -> Self::Future {
        let service_name = self.service_name.clone();
        let discover_fut = self.inner.subscribe(service_name);
        Box::pin(async move {
            discover_fut.await
        })
    }

}


impl<T, R> NewService<T> for R 
where
    T: Param<RpcInvocation>, 
    R: Registry + Clone,
    R::Discover: Discover + Unpin + Send,
{
    type Service = DiscoverService<R>;

    fn new_service(&self, target: T) -> Self::Service {
        let service_name = target.param().get_target_service_unique_name();

        DiscoverService {
            inner: self.clone(),
            service_name,
        }
    }
}



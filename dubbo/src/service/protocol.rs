use super::invocation;
use crate::common::url::Url;

use async_trait::async_trait;

#[async_trait]
pub trait Protocol {
    type Invoker;
    type Exporter;
    
    fn destroy(&self);
    async fn export(self, url: Url) -> Self::Exporter;
    async fn refer(&self, url: Url) -> Self::Invoker;
}


pub trait Exporter {
    type InvokerType: Invoker;

    fn unexport(&self);
    fn get_invoker(&self) -> Self::InvokerType;
}

pub trait Invoker {
    fn invoke<M1>(&self, req: invocation::Request<M1>) -> invocation::Response<String>
    where
        M1: Send + 'static;
    fn is_available(&self) -> bool;
    fn destroy(&self);
    fn get_url(&self) -> Url;
}
use std::{task::Poll, marker::PhantomData};

use dubbo_base::{svc::NewService, param::Param};
use futures_util::future;
use http::Request;
use tower::{ServiceExt, retry::Retry, util::Oneshot};
use tower_service::Service;

use crate::{codegen::RpcInvocation, StdError};

pub struct Failover<N> {
    inner: N // loadbalancer service
}

#[derive(Clone)]
pub struct FailoverPolicy;


pub struct NewFailover<N, Req> {
    inner: N, // new loadbalancer service
    _mark: PhantomData<Req>
}


impl<N, Req> NewFailover<N, Req> {
 
    pub fn new(inner: N) -> Self {
        NewFailover {
            inner,
            _mark: PhantomData,
        }
    }
} 

impl<N, T, Req> NewService<T> for NewFailover<N, Req> 
where
    T: Param<RpcInvocation>,
    // new loadbalancer service
    N: NewService<T>,
    // loadbalancer service
    N::Service: Service<Req> + Send + Clone + 'static, 
    <N::Service as Service<Req>>::Future: Send,
    <N::Service as Service<Req>>::Error: Into<StdError> + Send + Sync
{

    type Service = Failover<N::Service>;

    fn new_service(&self, target: T) -> Self::Service {
        // loadbalancer service
        let inner = self.inner.new_service(target);

        Failover {
            inner
        }
    } 

}
 

impl<B, Res, E> tower::retry::Policy<Request<B>, Res, E> for FailoverPolicy
where
    B: http_body::Body + Clone,
{
    
    type Future = future::Ready<Self>;

    fn retry(&self, req: &Request<B>, result: Result<&Res, &E>) -> Option<Self::Future> {
        //TODO some error handling or logging
        match result {
            Ok(_) => None,
            Err(_) => Some(future::ready(self.clone()))
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
    N: Service<Request<B>> + Clone + 'static ,
    N::Error: Into<StdError>,
    N::Future: Send
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
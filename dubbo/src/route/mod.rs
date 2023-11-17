use std::{sync::{Arc, Mutex}, collections::HashMap};

use futures_core::{Future, ready};
use futures_util::future::Ready;
use pin_project::pin_project;
use tokio::{sync::watch, pin};
use tower::{util::FutureService, buffer::Buffer};
use tower_service::Service;
  
use crate::{StdError, codegen::RpcInvocation, svc::NewService, param::Param, invocation::Invocation};

pub struct NewRoutes<N> {
    inner: N,
} 

pub struct NewRoutesCache<N> 
where
    N: NewService<RpcInvocation>
{
    inner: N,
    cache: Arc<Mutex<HashMap<String, N::Service>>>,
}

#[pin_project]
pub struct NewRoutesFuture<N, T> {
    #[pin]
    inner: N,
    target: T,
}
 
#[derive(Clone)]
pub struct Routes<Nsv, T> {
    target: T,
    new_invokers: Vec<Nsv>,
    invokers_receiver: watch::Receiver<Vec<Nsv>>,
}

impl<N> NewRoutes<N> {
    pub fn new(inner: N) -> Self {
        Self {
            inner, 
        }
    }
}


impl<N, T, Nsv> NewService<T> for NewRoutes<N> 
where
    T: Param<RpcInvocation> + Clone + Send + 'static, 
    // NewDirectory
    N: NewService<T>,
    // Directory
    N::Service: Service<(), Response = watch::Receiver<Vec<Nsv>>> + Unpin + Send + 'static,
    <N::Service as Service<()>>::Error: Into<StdError>,
    // new invoker service
    Nsv: NewService<()> + Clone + Send + Sync + 'static,
{

    type Service = Buffer<FutureService<NewRoutesFuture<<N as NewService<T>>::Service, T>, Routes<Nsv, T>>, ()>;

    fn new_service(&self, target: T) -> Self::Service {
        let inner = self.inner.new_service(target.clone());
        
        Buffer::new(FutureService::new(NewRoutesFuture {
            inner,
            target,
        }), 1024)
    }
}
impl<N, Nsv> NewRoutesCache<N> 
where
    N: NewService<RpcInvocation>,
    <N as NewService<RpcInvocation>>::Service: Service<(), Response = watch::Receiver<Vec<Nsv>>> + Unpin + Send + 'static,
    <N::Service as Service<()>>::Error: Into<StdError>,
    Nsv: NewService<()> + Clone + Send + Sync + 'static,

{
    pub fn layer() -> impl tower_layer::Layer<N, Service = NewRoutesCache<NewRoutes<N>>> {
        tower_layer::layer_fn(|inner: N| {
            NewRoutesCache::new(NewRoutes::new(inner))
        })
    }


}


impl<N> NewRoutesCache<N> 
where
    N: NewService<RpcInvocation>
{
    pub fn new(inner: N) -> Self {
        Self {
            inner,
            cache: Default::default(),
        }
    }
}

impl<N, T> NewService<T> for NewRoutesCache<N> 
where
    T: Param<RpcInvocation>,
    N: NewService<RpcInvocation>,
    N::Service: Clone,
{
    type Service = N::Service;

    fn new_service(&self, target: T) -> Self::Service {
        let rpc_inv = target.param();
        let service_name = rpc_inv.get_target_service_unique_name();

        let mut cache = self.cache.lock().expect("RoutesCache get lock failed");

        let service = cache.get(&service_name);
        match service {
            Some(service) => service.clone(),
            None => {
                let service = self.inner.new_service(rpc_inv);
                cache.insert(service_name, service.clone());
                service
            }
        }
    }
}


impl<N, T, Nsv> Future for NewRoutesFuture<N, T> 
where
    T: Param<RpcInvocation> + Clone, 
    // Directory
    N: Service<(), Response = watch::Receiver<Vec<Nsv>>> + Unpin,
    N::Error: Into<StdError>,
     // new invoker service
    Nsv: NewService<()> + Clone,
{
    type Output = Result<Routes<Nsv, T>, StdError>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {

        let this = self.get_mut();

        let target = this.target.clone();

        let _ = ready!(this.inner.poll_ready(cx)).map_err(Into::into)?;


        let call = this.inner.call(());
        pin!(call);
    
        let mut invokers_receiver = ready!(call.poll(cx).map_err(Into::into))?;
        let new_invokers = {
            let wait_for =  invokers_receiver.wait_for(|invs|!invs.is_empty());
            pin!(wait_for);
    
            let changed = ready!(wait_for.poll(cx))?;
    
            changed.clone()
        };
         

        std::task::Poll::Ready(Ok(Routes {
            invokers_receiver,
            new_invokers,
            target,
        }))
    }
}



impl<Nsv,T> Service<()> for Routes<Nsv, T> 
where
    T: Param<RpcInvocation> + Clone, 
     // new invoker service
    Nsv: NewService<()> + Clone,
{ 
    type Response = Vec<Nsv>;
 
    type Error = StdError;
  
    type Future = Ready<Result<Self::Response, Self::Error>>; 

    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        let has_change = self.invokers_receiver.has_changed()?;
        if has_change {
            self.new_invokers = self.invokers_receiver.borrow_and_update().clone();
        }
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: ()) -> Self::Future {
        // some router operator
        // if new_invokers changed, send new invokers to routes_rx after router operator
        futures_util::future::ok(self.new_invokers.clone()) 
    }
}
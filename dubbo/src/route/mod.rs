use dubbo_base::{svc::NewService, param::Param};
use futures_core::{Future, ready};
use futures_util::future::Ready;
use pin_project::pin_project;
use tokio::{sync::watch, pin};
use tower::util::FutureService;
use tower_service::Service;
  
use crate::{StdError, codegen::RpcInvocation};

pub struct NewRoutes<N> {
    inner: N,
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

    pub fn layer() -> impl tower_layer::Layer<N, Service = Self> {
        tower_layer::layer_fn(|inner: N| {
            NewRoutes {
                inner, // NewDirectory
            }
        })
    }
}


impl<N, T, Nsv> NewService<T> for NewRoutes<N> 
where
    T: Param<RpcInvocation> + Clone, 
    // NewDirectory
    N: NewService<T>,
    // Directory
    N::Service: Service<(), Response = watch::Receiver<Vec<Nsv>>>,
    // new invoker service
    Nsv: NewService<()> + Clone
{

    type Service = FutureService<NewRoutesFuture<N::Service, T>, Routes<Nsv, T>>;

    fn new_service(&self, target: T) -> Self::Service {
        let inner = self.inner.new_service(target.clone());
        
        FutureService::new(NewRoutesFuture {
            inner,
            target,
        })
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
use std::{task::Poll, pin::Pin, mem};

use dubbo_logger::tracing::debug;
use futures_core::{Future, TryFuture, ready, future::BoxFuture};
use futures_util::FutureExt;
use pin_project::pin_project;
use thiserror::Error;
use tokio::{task::JoinHandle, sync::{watch::{Sender, Receiver}, self}};
use tokio_util::sync::ReusableBoxFuture;
use tower::{ServiceExt, buffer::Buffer};
use tower_service::Service;

use crate::StdError;

enum Inner<S> {
    Invalid,
    Ready(S),
    Pending(JoinHandle<Result<S, (S, StdError)>>),
} 

#[derive(Debug, Error)]
#[error("the inner service has not got ready yet!")]
struct InnerServiceNotReadyErr;



#[pin_project(project = InnerServiceCallingResponseProj)]
enum InnerServiceCallingResponse<Fut> {
    Call(#[pin] Fut),
    Fail
}

impl<Fut> Future for InnerServiceCallingResponse<Fut> 
where
    Fut: TryFuture,
    Fut::Error: Into<StdError>
{
    type Output = Result<Fut::Ok, StdError>;
 
    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
       match self.project() {
        InnerServiceCallingResponseProj::Call(call) => call.try_poll(cx).map_err(Into::into),
        InnerServiceCallingResponseProj::Fail => Poll::Ready(Err(InnerServiceNotReadyErr.into()))
       }
    }
}
 
#[derive(Clone)]
enum ObserveState {
    Ready,
    Pending,
}


struct ReadyService<S> {
    inner: Inner<S>,
    tx: Sender<ObserveState>
}


impl<S> ReadyService<S> {

    fn new(inner: S) -> (Self, Receiver<ObserveState>) {
        let (tx, rx) = sync::watch::channel(ObserveState::Ready);
        let ready_service = Self { inner: Inner::Ready(inner), tx};
        (ready_service, rx)
    }

}

impl<S, Req> Service<Req> for ReadyService<S> 
where
    S: Service<Req> + Send + 'static,
    <S as Service<Req>>::Error: Into<StdError>,
{
    type Response = S::Response;

    type Error = StdError;

    type Future = InnerServiceCallingResponse<S::Future>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        loop {
            match mem::replace(&mut self.inner, Inner::Invalid) {
                Inner::Ready(mut svc) => {
                    let poll_ready  = svc.poll_ready(cx);
                    match poll_ready {
                        Poll::Pending => {
                            self.inner = Inner::Pending(tokio::spawn(async move {
                                let poll_ready = svc.ready().await;
                                match poll_ready {
                                    Ok(_) => Ok(svc),
                                    Err(err) => {
                                        Err((svc, err.into()))
                                    }
                                }
                            }));

                            let _ = self.tx.send(ObserveState::Pending);
                            continue;
                        }
                        Poll::Ready(ret) => {
                            self.inner = Inner::Ready(svc);

                            let _ = self.tx.send(ObserveState::Ready);
                            return Poll::Ready(ret.map_err(Into::into));
                        }
                    }
                },
                Inner::Pending(mut join_handle) => {
                    if let Poll::Ready(res) = join_handle.poll_unpin(cx) {
                        let (svc, res) = match res {
                            Err(join_err) => panic!("ReadyService panicked: {join_err}"),
                            Ok(Err((svc, err))) => (svc, Poll::Ready(Err(err))),
                            Ok(Ok(svc)) => (svc, Poll::Ready(Ok(())))
                        };

                        self.inner = Inner::Ready(svc);
                        
                        let _ = self.tx.send(ObserveState::Ready);
                        return res;
                    } else {
                        self.inner = Inner::Pending(join_handle);

                        let _ = self.tx.send(ObserveState::Pending);
                        return Poll::Pending;
                    }
                    
                },
                Inner::Invalid => panic!("ReadyService panicked: inner state is invalid")
            }
        }
    }

    fn call(&mut self, req: Req) -> Self::Future {
       match self.inner {
        Inner::Ready(ref mut svc) => InnerServiceCallingResponse::Call(svc.call(req)),
        _ => InnerServiceCallingResponse::Fail
       }
    }
}


impl<S> Drop for ReadyService<S> {
    fn drop(&mut self) {
       if let Inner::Pending(ref handler) = self.inner {
            handler.abort();
       }
    }
}

pub struct CloneInvoker<Inv, Req> 
where
    Inv: Service<Req> + Send + 'static,
    Inv::Error: Into<StdError> + Send + Sync + 'static,
    Inv::Future: Send,
    Req: Send
{
    inner: Buffer<ReadyService<Inv>, Req>,
    rx: Receiver<ObserveState>,
    poll: ReusableBoxFuture<'static, ObserveState>,
    polling: bool,
}

impl<Inv, Req> CloneInvoker<Inv, Req> 
where
    Inv: Service<Req> + Send + 'static,
    Inv::Error: Into<StdError> + Send + Sync + 'static,
    Inv::Future: Send,
    Req: Send + 'static
{

    pub fn new(invoker: Inv) -> Self {
        
        let (ready_service, rx) = ReadyService::new(invoker);

        let buffer: Buffer<ReadyService<Inv>, Req> = Buffer::new(ready_service, 1024);

        Self { inner: buffer, rx, polling: false, poll: ReusableBoxFuture::new(futures::future::pending()) }
    }
}

impl<Inv, Req> Service<Req> for CloneInvoker<Inv, Req> 
where
    Inv: Service<Req> + Send + 'static,
    Inv::Error: Into<StdError> + Send + Sync + 'static,
    Inv::Future: Send,
    Req: Send + 'static
{
    type Response = Inv::Response;
 
    type Error = StdError;
 
    type Future = BoxFuture<'static, Result<Self::Response, StdError>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        loop {
            if !self.polling { 
                match self.rx.borrow().clone() {
                    ObserveState::Ready => return self.inner.poll_ready(cx),
                    ObserveState::Pending => {
                        self.polling = true;
                        let mut rx = self.rx.clone();
                        self.poll.set(async move {
                            loop {
                                let current_state = rx.borrow_and_update().clone();
                                if matches!(current_state, ObserveState::Ready) {
                                    return current_state;                            
                                }
                                if let Err(_) = rx.changed().await {
                                    debug!("the readyService has already shutdown!");
                                    futures::future::pending::<ObserveState>().await;
                                }
                            }
                        });
                    }
                }
    
            }

            let state = ready!(self.poll.poll_unpin(cx));
            self.polling = false;

            if matches!(state, ObserveState::Pending) {
                continue;
            }

            return self.inner.poll_ready(cx);
        }
    }

    fn call(&mut self, req: Req) -> Self::Future {
        Box::pin(self.inner.call(req))
    }
}


impl<Inv, Req> Clone for CloneInvoker<Inv, Req> 
where
    Inv: Service<Req> + Send + 'static,
    Inv::Error: Into<StdError> + Send + Sync + 'static,
    Inv::Future: Send,
    Req: Send
{
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone(), rx: self.rx.clone(), polling: false, poll: ReusableBoxFuture::new(futures::future::pending())}
    }
}  
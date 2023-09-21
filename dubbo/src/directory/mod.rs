/*
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.
 * The ASF licenses this file to You under the Apache License, Version 2.0
 * (the "License"); you may not use this file except in compliance with
 * the License.  You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

 use core::panic;
use std::{
    hash::Hash,
    pin::Pin,
    task::{Context, Poll}, collections::HashMap, sync::{Arc, Mutex}, marker::PhantomData,
};
    
use crate::{StdError, codegen::RpcInvocation, invocation::Invocation};
use dubbo_base::{param::Param, svc::NewService};
use futures_core::Future;
use futures_util::future::{poll_fn, self};
use pin_project::pin_project;
use tokio::{sync::{watch, Notify}, select};
use tower::{
    discover::{Change, Discover}, ServiceExt, buffer::Buffer, util::{FutureService, Oneshot},
};

use tower_service::Service;

pub struct NewCachedDirectory<T>(PhantomData<T>);


pub struct CachedDirectory<N, T> 
where
    // NewDirectory 
    N: NewService<T>
{
    inner: N,
    cache: Arc<Mutex<HashMap<String, N::Service>>>
}


pub struct NewDirectory<N> {
    // service registry
    inner: N,
}

 
#[pin_project]
pub struct NewDirectoryFuture<S> 
where
    S: Service<()>,
    S::Response: Discover
{
    #[pin]
    inner: Oneshot<S, ()>
}

#[derive(Clone)]
pub struct Directory<D> 
where
    D: Discover
{
    rx: watch::Receiver<Vec<D::Service>>,
    close: Arc<Notify>
}

impl<T> NewCachedDirectory<T> {

    pub fn new() -> Self {
        NewCachedDirectory(PhantomData)  
    }
}


impl<N, T> NewService<N> for NewCachedDirectory<T>
where
    T: Param<RpcInvocation>,
    // service registry
    N: NewService<T>, 
    N::Service: Service<()> + Send,
    // Discover service
    <N::Service as Service<()>>::Response: Discover + Unpin + Send,
    <N::Service as Service<()>>::Error: Into<StdError>, 
    <N::Service as Service<()>>::Future: Unpin + Send, 
    // Discover::Key
    <<N::Service as Service<()>>::Response as Discover>::Key: Hash + Eq + Clone + Send,
    // Discover::Service = new invoker
    <<N::Service as Service<()>>::Response as Discover>::Service: NewService<()> + Clone + Send + Sync,
{
    type Service = CachedDirectory<NewDirectory<N>, T>;

    fn new_service(&self, target: N) -> Self::Service {
        
        CachedDirectory::new(NewDirectory::new(target))
    }
} 
 

impl<N, T> CachedDirectory<N, T> 
where
    N: NewService<T>
{

    pub fn new(inner: N) -> Self {
        CachedDirectory {
            inner,
            cache: Default::default()
        }
    } 
} 


impl<N, T> NewService<T> for CachedDirectory<N, T> 
where
    T: Param<RpcInvocation>,
    // NewDirectory
    N: NewService<T>,
    // Buffered directory
    N::Service: Clone
{
    type Service = N::Service;

    fn new_service(&self, target: T) -> Self::Service {
        let rpc_invocation = target.param();
        let service_name = rpc_invocation.get_target_service_unique_name();
        let mut cache = self.cache.lock().expect("cached directory lock failed.");
        let value = cache.get(&service_name).map(|val|val.clone());
        match value {
            None => {
                let new_service = self.inner.new_service(target);
                cache.insert(service_name, new_service.clone());
                new_service
            },
            Some(value) => value
        } 
    }
}


impl<N> NewDirectory<N> {

    pub fn new(inner: N) -> Self {
        NewDirectory {
            inner
        }
    }
}

impl<N, T> NewService<T> for NewDirectory<N> 
where
    T: Param<RpcInvocation>,
    // service registry
    N: NewService<T>, 
    N::Service: Service<()> + Send, 
    // Discover service
    <N::Service as Service<()>>::Response: Discover + Unpin + Send,
    <N::Service as Service<()>>::Future: Unpin + Send,
    <N::Service as Service<()>>::Error: Into<StdError>,
    // Discover::Key
    <<N::Service as Service<()>>::Response as Discover>::Key: Hash + Eq + Clone + Send,
    // Discover::service = new invoker
    <<N::Service as Service<()>>::Response as Discover>::Service: Send + Sync + Clone + NewService<()>,
{
    type Service = Buffer<FutureService<NewDirectoryFuture<<N as NewService<T>>::Service>, Directory<<N::Service as Service<()>>::Response>>, ()>; 
 
    fn new_service(&self, target: T) -> Self::Service {
        let discovery_service =  self.inner.new_service(target);  
        Buffer::new(FutureService::new(NewDirectoryFuture::new(discovery_service)), 10)
    } 

} 



impl<S> NewDirectoryFuture<S> 
where
    // Discover service
    S: Service<()>,
    S::Response: Discover
{
    pub fn new(inner: S) -> Self {
        NewDirectoryFuture {
            inner: inner.oneshot(())
        }
    }
}



impl<S> Future for NewDirectoryFuture<S> 
where
    // Discover service
    S: Service<()>,
    // Discover
    S::Response: Discover + Unpin + Send,
    // the key may be dubbo url
    <S::Response as Discover>::Key: Hash + Eq + Clone + Send, 
    // error
    S::Error: Into<StdError>, 
    // new invoker
    <S::Response as Discover>::Service: NewService<()> + Clone + Send + Sync
{
    type Output = Result<Directory<S::Response>, StdError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project(); 

        this.inner.poll(cx).map(|poll_ret| {
            poll_ret.map(|discover| { 
                Directory::new(discover)
            })
        }).map_err(|e| {
            e.into()
        }) 

    }
}

impl<D> Directory<D> 
where
    // Discover
    D: Discover + Unpin + Send,
    // the key may be dubbo url
    D::Key: Hash + Eq + Clone + Send,
    // invoker new service
    D::Service: NewService<()> + Clone + Send + Sync,
{

    pub fn new(mut discover: D) -> Self {

        let pin_discover = Pin::new(&mut discover);
        let (tx, rx) = watch::channel(Vec::new());
        let close = Arc::new(Notify::new());
        let close_clone = close.clone();
     
        tokio::spawn(async move {
            let mut cache: HashMap<D::Key, D::Service> = HashMap::new();

            loop {
                let changed = select! {
                    _ = close_clone.notified() => {
                        // info!("discover stream closed.")
                        return;
                    },
                    changed = poll_fn(|cx| pin_discover.poll_discover(cx)) => {
                        changed
                    }
                };
                let Some(changed) = changed else {
                    // debug!("discover stream closed.");
                    break;
                };

                match changed {
                    Err(e) => {
                        // error!("discover stream error: {}", e);
                        continue;
                    },
                    Ok(changed) => match changed {
                        Change::Insert(k, v) => {
                            cache.insert(k, v);
                        },
                        Change::Remove(k) => {
                            cache.remove(&k);
                        }
                    }
                }

                let vec: Vec<D::Service> = cache.values().map(|v|v.clone()).collect();
                let _ = tx.send(vec);
            }
            
        });
        Directory {
            rx,
            close
        }
    }
}


impl<D> Service<()> for Directory<D> 
where
    // Discover
    D: Discover + Unpin + Send,
    // the key may be dubbo url
    D::Key: Hash + Eq + Clone + Send,
    // invoker new service
    D::Service: NewService<()> + Clone + Send + Sync,
{ 
    type Response = watch::Receiver<Vec<D::Service>>;

    type Error = StdError;

    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: ()) -> Self::Future {
        future::ok(self.rx.clone())
    }
}

impl<D> Drop for Directory<D> 
where
    D: Discover, 
{ 
    fn drop(&mut self) {
        if Arc::strong_count(&self.close) == 1 {
            self.close.notify_one();
        }
    }
}
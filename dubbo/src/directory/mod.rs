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
    task::{Context, Poll}, collections::HashMap, sync::{Arc, Mutex},
};
    
use crate::{StdError, codegen::RpcInvocation, invocation::Invocation, registry::n_registry::Registry, invoker::NewInvoker, svc::NewService, param::Param};
use futures_util::future::{poll_fn, self};
use tokio::{sync::{watch, Notify, mpsc::channel}, select};
use tokio_stream::wrappers::ReceiverStream;
use tower::{
    discover::{Change, Discover}, buffer::Buffer,
};

use tower_service::Service;

type BufferedDirectory = Buffer<Directory<ReceiverStream<Result<Change<String, NewInvoker>, StdError>>>, ()>;

pub struct NewCachedDirectory<N>
where
    N: Registry + Clone + Send + Sync + 'static,
{
    inner: CachedDirectory<NewDirectory<N>, RpcInvocation>
}


pub struct CachedDirectory<N, T> 
where
    // NewDirectory 
    N: NewService<T>
{
    inner: N,
    cache: Arc<Mutex<HashMap<String, N::Service>>>
}


pub struct NewDirectory<N> {
    // registry
    inner: N,
}



#[derive(Clone)]
pub struct Directory<D> 
where
    D: Discover
{
    rx: watch::Receiver<Vec<D::Service>>,
    close: Arc<Notify>
}



impl<N> NewCachedDirectory<N> 
where
    N: Registry + Clone + Send + Sync + 'static,
{

    pub fn layer() -> impl tower_layer::Layer<N, Service = Self> {
        tower_layer::layer_fn(|inner: N| {
            NewCachedDirectory {
                // inner is registry 
                inner: CachedDirectory::new(NewDirectory::new(inner)), 
            }
        })
    }
}


impl<N, T> NewService<T> for NewCachedDirectory<N>
where
    T: Param<RpcInvocation>,
    // service registry
    N: Registry + Clone + Send + Sync + 'static,
{
    type Service = BufferedDirectory; 

    fn new_service(&self, target: T) -> Self::Service {
    
        self.inner.new_service(target.param())
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
    N: Registry + Clone + Send + Sync + 'static, 
{
    type Service = BufferedDirectory; 
 
    fn new_service(&self, target: T) -> Self::Service {
        
        let service_name = target.param().get_target_service_unique_name();

        let registry = self.inner.clone();

        let (tx, rx) = channel(1024);

        tokio::spawn(async move {

            let receiver = registry.subscribe(service_name).await;
            match receiver {
                Err(e) => {
                    // error!("discover stream error: {}", e);
                    
                },
                Ok(mut receiver) => {
                    loop {
                        let change = receiver.recv().await;
                        match change {
                            None => {
                                // debug!("discover stream closed.");
                                break;
                            },
                            Some(change) => {
                                let _  = tx.send(change);
                            }
                        }
                    }
                }
            }

        }); 

        Buffer::new(Directory::new(ReceiverStream::new(rx)), 1024)
    } 

} 


impl<D> Directory<D> 
where
    // Discover
    D: Discover + Unpin + Send + 'static,
    // the key may be dubbo url
    D::Key: Hash + Eq + Clone + Send,
    // invoker new service
    D::Service: NewService<()> + Clone + Send + Sync,
{ 

    pub fn new(discover: D) -> Self {
  
        let mut discover = Box::pin(discover);

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
                    changed = poll_fn(|cx| discover.as_mut().poll_discover(cx)) => {
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
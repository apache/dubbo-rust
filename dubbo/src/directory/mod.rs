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

use std::{
    collections::HashMap,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use crate::{
    codegen::{RpcInvocation, TripleInvoker},
    invocation::Invocation,
    invoker::{clone_invoker::CloneInvoker, NewInvoker},
    param::Param,
    registry::n_registry::Registry,
    svc::NewService,
    StdError,
};
use dubbo_base::Url;
use dubbo_logger::tracing::debug;
use futures_core::ready;
use futures_util::future;
use tokio::sync::mpsc::channel;
use tokio_stream::wrappers::ReceiverStream;
use tower::{
    buffer::Buffer,
    discover::{Change, Discover},
};

use tower_service::Service;

type BufferedDirectory =
    Buffer<Directory<ReceiverStream<Result<Change<String, ()>, StdError>>>, ()>;

pub struct NewCachedDirectory<N>
where
    N: Registry + Clone + Send + Sync + 'static,
{
    inner: CachedDirectory<NewDirectory<N>, RpcInvocation>,
}

pub struct CachedDirectory<N, T>
where
    // NewDirectory
    N: NewService<T>,
{
    inner: N,
    cache: Arc<Mutex<HashMap<String, N::Service>>>,
}

pub struct NewDirectory<N> {
    // registry
    inner: N,
}

pub struct Directory<D> {
    directory: HashMap<String, CloneInvoker<TripleInvoker>>,
    discover: D,
    new_invoker: NewInvoker,
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
    N: NewService<T>,
{
    pub fn new(inner: N) -> Self {
        CachedDirectory {
            inner,
            cache: Default::default(),
        }
    }
}

impl<N, T> NewService<T> for CachedDirectory<N, T>
where
    T: Param<RpcInvocation>,
    // NewDirectory
    N: NewService<T>,
    // Buffered directory
    N::Service: Clone,
{
    type Service = N::Service;

    fn new_service(&self, target: T) -> Self::Service {
        let rpc_invocation = target.param();
        let service_name = rpc_invocation.get_target_service_unique_name();
        let mut cache = self.cache.lock().expect("cached directory lock failed.");
        let value = cache.get(&service_name).map(|val| val.clone());
        match value {
            None => {
                let new_service = self.inner.new_service(target);
                cache.insert(service_name, new_service.clone());
                new_service
            }
            Some(value) => value,
        }
    }
}

impl<N> NewDirectory<N> {
    const MAX_DIRECTORY_BUFFER_SIZE: usize = 16;

    pub fn new(inner: N) -> Self {
        NewDirectory { inner }
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

        let (tx, rx) = channel(Self::MAX_DIRECTORY_BUFFER_SIZE);

        tokio::spawn(async move {
            // todo use dubbo url model generate subscribe url
            // category:serviceInterface:version:group
            let consumer_url = format!("consumer://{}/{}", "127.0.0.1:8888", service_name);
            let subscribe_url = Url::from_url(&consumer_url).unwrap();
            let receiver = registry.subscribe(subscribe_url).await;
            debug!("discover start!");
            match receiver {
                Err(_e) => {
                    // error!("discover stream error: {}", e);
                    debug!("discover stream error");
                }
                Ok(mut receiver) => loop {
                    let change = receiver.recv().await;
                    debug!("receive change: {:?}", change);
                    match change {
                        None => {
                            debug!("discover stream closed.");
                            break;
                        }
                        Some(change) => {
                            let _ = tx.send(change).await;
                        }
                    }
                },
            }
        });

        Buffer::new(
            Directory::new(ReceiverStream::new(rx)),
            Self::MAX_DIRECTORY_BUFFER_SIZE,
        )
    }
}

impl<D> Directory<D> {
    pub fn new(discover: D) -> Self {
        Directory {
            directory: Default::default(),
            discover,
            new_invoker: NewInvoker,
        }
    }
}

impl<D> Service<()> for Directory<D>
where
    // Discover
    D: Discover<Key = String> + Unpin + Send,
    D::Error: Into<StdError>,
{
    type Response = Vec<CloneInvoker<TripleInvoker>>;

    type Error = StdError;

    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        loop {
            let pin_discover = Pin::new(&mut self.discover);

            match pin_discover.poll_discover(cx) {
                Poll::Pending => {
                    if self.directory.is_empty() {
                        return Poll::Pending;
                    } else {
                        return Poll::Ready(Ok(()));
                    }
                }
                Poll::Ready(change) => {
                    let change = change.transpose().map_err(|e| e.into())?;
                    match change {
                        Some(Change::Remove(key)) => {
                            debug!("remove key: {}", key);
                            self.directory.remove(&key);
                        }
                        Some(Change::Insert(key, _)) => {
                            debug!("insert key: {}", key);
                            let invoker = self.new_invoker.new_service(key.clone());
                            self.directory.insert(key, invoker);
                        }
                        None => {
                            debug!("stream closed");
                            return Poll::Ready(Ok(()));
                        }
                    }
                }
            }
        }
    }

    fn call(&mut self, _: ()) -> Self::Future {
        let vec = self
            .directory
            .values()
            .map(|val| val.clone())
            .collect::<Vec<CloneInvoker<TripleInvoker>>>();
        future::ok(vec)
    }
}

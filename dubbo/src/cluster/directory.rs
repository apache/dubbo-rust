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
    fmt::Debug,
    pin::Pin,
    str::FromStr,
    sync::{Arc, RwLock},
    task::{Context, Poll},
};

use crate::{
    codegen::TripleInvoker,
    protocol::BoxInvoker,
    registry::{memory_registry::MemoryNotifyListener, BoxRegistry},
    triple::client::replay::ClonedBody,
    StdError,
};
use dubbo_base::Url;
use dubbo_logger::tracing;
use futures_core::ready;
use tower::{
    discover::{Change, Discover},
    ready_cache::ReadyCache,
};

use crate::{cluster::Directory, codegen::RpcInvocation, invocation::Invocation};

/// Directory.
///
/// [Directory Service](http://en.wikipedia.org/wiki/Directory_service)

#[derive(Debug, Clone)]
pub struct StaticDirectory {
    uri: http::Uri,
}

impl StaticDirectory {
    pub fn new(host: &str) -> StaticDirectory {
        let uri = match http::Uri::from_str(host) {
            Ok(v) => v,
            Err(err) => {
                tracing::error!("http uri parse error: {}, host: {}", err, host);
                panic!("http uri parse error: {}, host: {}", err, host)
            }
        };
        StaticDirectory { uri }
    }

    pub fn from_uri(uri: &http::Uri) -> StaticDirectory {
        StaticDirectory { uri: uri.clone() }
    }
}

impl Directory for StaticDirectory {
    fn list(&self, inv: Arc<RpcInvocation>) -> Vec<BoxInvoker> {
        let url = Url::from_url(&format!(
            "tri://{}:{}/{}",
            self.uri.host().unwrap(),
            self.uri.port().unwrap(),
            inv.get_target_service_unique_name(),
        ))
        .unwrap();
        let invoker = Box::new(TripleInvoker::new(url));
        vec![invoker]
    }
}

#[derive(Debug, Clone)]
pub struct RegistryDirectory {
    registry: Arc<BoxRegistry>,
    service_instances: Arc<RwLock<HashMap<String, Vec<Url>>>>,
}

impl RegistryDirectory {
    pub fn new(registry: BoxRegistry) -> RegistryDirectory {
        RegistryDirectory {
            registry: Arc::new(registry),
            service_instances: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Directory for RegistryDirectory {
    fn list(&self, inv: Arc<RpcInvocation>) -> Vec<BoxInvoker> {
        let service_name = inv.get_target_service_unique_name();
        let url = Url::from_url(&format!(
            "triple://{}:{}/{}",
            "127.0.0.1", "8888", service_name
        ))
        .unwrap();

        self.registry
            .subscribe(
                url,
                Arc::new(MemoryNotifyListener {
                    service_instances: Arc::clone(&self.service_instances),
                }),
            )
            .expect("subscribe");

        let map = self
            .service_instances
            .read()
            .expect("service_instances.read");
        let binding = Vec::new();
        let url_vec = map.get(&service_name).unwrap_or(&binding);
        // url_vec.to_vec()
        let mut invokers: Vec<BoxInvoker> = vec![];
        for item in url_vec.iter() {
            invokers.push(Box::new(TripleInvoker::new(item.clone())));
        }

        invokers
    }
}

pub struct ServiceNameDirectory<D> {
    cache: ReadyCache<String, BoxInvoker, http::Request<ClonedBody>>,
    discover: D,
}

impl<D> ServiceNameDirectory<D>
where
    D: Discover<Key = String, Service = BoxInvoker> + Unpin,
    D::Error: Into<StdError>,
{
    pub fn new(discover: D) -> Self {
        Self {
            cache: ReadyCache::default(),
            discover,
        }
    }

    fn update_cache(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), StdError>> {
        loop {
            let discover = Pin::new(&mut self.discover);
            let update = ready!(discover.poll_discover(cx));
            match update {
                Some(update) => {
                    let change = match update {
                        Err(_) => continue,
                        Ok(change) => change,
                    };

                    match change {
                        Change::Insert(key, invoker) => {
                            self.cache.push(key, invoker);
                        }
                        Change::Remove(key) => {
                            self.cache.evict(&key);
                        }
                    }
                }
                None => break,
            }
        }

        // poll pending
        let _ = ready!(self.cache.poll_pending(cx))?;

        Poll::Ready(Ok(()))
    }

    pub fn list(&mut self, cx: &mut Context<'_>) -> Poll<Result<Vec<String>, StdError>> {
        let _ = self.update_cache(cx)?;

        let ready_len = self.cache.ready_len();

        if ready_len == 0 {
            return Poll::Pending;
        }

        let mut invoker_list = Vec::with_capacity(ready_len);

        for idx in 0..ready_len {
            let check = self.cache.check_ready_index(cx, idx);
            let is_ready = match check {
                Ok(is_ready) => is_ready,
                Err(_) => false,
            };
            if !is_ready {
                continue;
            }

            let invoker_url = match self.cache.get_ready_index(idx) {
                None => continue,
                Some((k, _)) => k.clone(),
            };

            invoker_list.push(invoker_url);
        }

        Poll::Ready(Ok(invoker_list))
    }
}

#[cfg(test)]
pub mod tests {
    use std::{
        convert::Infallible,
        pin::Pin,
        task::{Context, Poll},
    };

    use crate::{boxed, cluster::directory::ServiceNameDirectory};
    use bytes::Buf;
    use futures_core::Stream;
    use futures_util::future::poll_fn;
    use http::StatusCode;
    use http_body::Body;
    use tower::{discover::Change, Service};

    use dubbo_base::Url;

    use crate::{codegen::Invoker, protocol::BoxInvoker, triple::client::replay::ClonedBody};

    pub struct MockStaticServiceList<T>
    where
        T: IntoIterator,
    {
        iter: T::IntoIter,
    }

    impl<T> MockStaticServiceList<T>
    where
        T: IntoIterator<Item = BoxInvoker>,
    {
        pub fn new(list: T) -> Self {
            let iter = list.into_iter();

            Self { iter }
        }
    }

    impl<T> Stream for MockStaticServiceList<T>
    where
        T: IntoIterator<Item = BoxInvoker>,
        T::IntoIter: Unpin,
    {
        type Item = Result<Change<String, BoxInvoker>, Infallible>;

        fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            let mut_self_ref = self.get_mut();
            let mut_iter_ref = &mut mut_self_ref.iter;

            match mut_iter_ref.next() {
                Some(next) => {
                    let invoker_url = next.get_url();
                    let raw_url_string = invoker_url.raw_url_string();
                    return Poll::Ready(Some(Ok(Change::Insert(raw_url_string, next))));
                }
                None => Poll::Ready(None),
            }
        }
    }

    #[derive(Debug)]
    struct MockInvoker(u8, bool);

    impl Invoker<http::Request<ClonedBody>> for MockInvoker {
        fn get_url(&self) -> dubbo_base::Url {
            let str = format!(
                "triple://127.0.0.1:8888/failover_cluster_service/directory/{}",
                self.0
            );
            Url::from_url(str.as_str()).unwrap()
        }
    }

    impl Service<http::Request<ClonedBody>> for MockInvoker {
        type Response = http::Response<crate::BoxBody>;

        type Error = crate::Error;

        type Future = crate::BoxFuture<http::Response<crate::BoxBody>, crate::Error>;

        fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            if !self.1 {
                return Poll::Pending;
            }

            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: http::Request<ClonedBody>) -> Self::Future {
            Box::pin(async move {
                let mut body = req.into_body();

                let mut body_data = poll_fn(|cx| Pin::new(&mut body).poll_data(cx))
                    .await
                    .unwrap()
                    .unwrap();

                let len = body_data.remaining();
                let bytes = body_data.copy_to_bytes(len);

                let req_str = String::from_utf8(bytes.to_vec()).unwrap();
                println!("req: {}", req_str);
                let echo_data = format!("echo: {}", req_str);
                let response = http::Response::builder()
                    .status(StatusCode::OK)
                    .body(boxed(echo_data))
                    .unwrap();

                return Ok(response);
            })
        }
    }

    #[tokio::test]
    async fn test_directory() {
        let invoker_list = vec![
            Box::new(MockInvoker(1, true)) as BoxInvoker,
            Box::new(MockInvoker(2, true)) as BoxInvoker,
            Box::new(MockInvoker(3, true)) as BoxInvoker,
            Box::new(MockInvoker(4, true)) as BoxInvoker,
            Box::new(MockInvoker(5, true)) as BoxInvoker,
            Box::new(MockInvoker(6, true)) as BoxInvoker,
        ];
        let service_list = MockStaticServiceList::new(invoker_list);

        let mut service_name_directory = ServiceNameDirectory::new(service_list);

        let list = poll_fn(|cx| service_name_directory.list(cx)).await;

        assert!(list.is_ok());

        let list = list.unwrap();

        assert!(!list.is_empty());

        assert_eq!(6, list.len());

        for uri in list {
            println!("invoker uri: {}", uri);
        }
    }

    #[tokio::test]
    async fn test_ready_in_any_order() {
        let invoker_list = vec![
            Box::new(MockInvoker(1, true)) as BoxInvoker,
            Box::new(MockInvoker(2, false)) as BoxInvoker,
            Box::new(MockInvoker(3, false)) as BoxInvoker,
            Box::new(MockInvoker(4, true)) as BoxInvoker,
            Box::new(MockInvoker(5, false)) as BoxInvoker,
            Box::new(MockInvoker(6, true)) as BoxInvoker,
        ];
        let service_list = MockStaticServiceList::new(invoker_list);

        let mut service_name_directory = ServiceNameDirectory::new(service_list);

        let list = poll_fn(|cx| service_name_directory.list(cx)).await;

        assert!(list.is_ok());

        let list = list.unwrap();

        assert!(!list.is_empty());

        assert_eq!(3, list.len());

        for uri in list {
            println!("invoker uri: {}", uri);
        }
    }
}

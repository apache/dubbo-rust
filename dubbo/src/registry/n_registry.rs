use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use async_trait::async_trait;
use dubbo_base::Url;
use thiserror::Error;
use tokio::sync::{
    mpsc::{self, Receiver},
    Mutex,
};
use tower::discover::Change;

use crate::StdError;

pub type ServiceChange = Change<String, ()>;
pub type DiscoverStream = Receiver<Result<ServiceChange, StdError>>;
pub type BoxRegistry = Box<dyn Registry + Send + Sync>;

#[async_trait]
pub trait Registry {
    async fn register(&self, url: Url) -> Result<(), StdError>;

    async fn unregister(&self, url: Url) -> Result<(), StdError>;

    async fn subscribe(&self, url: Url) -> Result<DiscoverStream, StdError>;

    async fn unsubscribe(&self, url: Url) -> Result<(), StdError>;
}

#[derive(Clone)]
pub struct ArcRegistry {
    inner: Arc<dyn Registry + Send + Sync + 'static>,
}

pub enum RegistryComponent {
    NacosRegistry(ArcRegistry),
    ZookeeperRegistry,
    StaticRegistry(StaticRegistry),
}

pub struct StaticServiceValues {
    listeners: Vec<mpsc::Sender<Result<ServiceChange, StdError>>>,
    urls: HashSet<String>,
}

#[derive(Default)]
pub struct StaticRegistry {
    urls: Mutex<HashMap<String, StaticServiceValues>>,
}

impl ArcRegistry {
    pub fn new(registry: impl Registry + Send + Sync + 'static) -> Self {
        Self {
            inner: Arc::new(registry),
        }
    }
}

#[async_trait]
impl Registry for ArcRegistry {
    async fn register(&self, url: Url) -> Result<(), StdError> {
        self.inner.register(url).await
    }

    async fn unregister(&self, url: Url) -> Result<(), StdError> {
        self.inner.unregister(url).await
    }

    async fn subscribe(&self, url: Url) -> Result<DiscoverStream, StdError> {
        self.inner.subscribe(url).await
    }

    async fn unsubscribe(&self, url: Url) -> Result<(), StdError> {
        self.inner.unsubscribe(url).await
    }
}

#[async_trait]
impl Registry for RegistryComponent {
    async fn register(&self, url: Url) -> Result<(), StdError> {
        todo!()
    }

    async fn unregister(&self, url: Url) -> Result<(), StdError> {
        todo!()
    }

    async fn subscribe(&self, url: Url) -> Result<DiscoverStream, StdError> {
        match self {
            RegistryComponent::NacosRegistry(registry) => registry.subscribe(url).await,
            RegistryComponent::ZookeeperRegistry => todo!(),
            RegistryComponent::StaticRegistry(registry) => registry.subscribe(url).await,
        }
    }

    async fn unsubscribe(&self, url: Url) -> Result<(), StdError> {
        todo!()
    }
}

impl StaticRegistry {
    pub fn new(urls: Vec<Url>) -> Self {
        let mut map = HashMap::with_capacity(urls.len());

        for url in urls {
            let service_name = url.get_service_name();
            let static_values = map
                .entry(service_name)
                .or_insert_with(|| StaticServiceValues {
                    listeners: Vec::new(),
                    urls: HashSet::new(),
                });
            let url = url.to_string();
            static_values.urls.insert(url.clone());
        }

        Self {
            urls: Mutex::new(map),
        }
    }
}

#[async_trait]
impl Registry for StaticRegistry {
    async fn register(&self, url: Url) -> Result<(), StdError> {
        let service_name = url.get_service_name();
        let mut lock = self.urls.lock().await;

        let static_values = lock
            .entry(service_name)
            .or_insert_with(|| StaticServiceValues {
                listeners: Vec::new(),
                urls: HashSet::new(),
            });
        let url = url.to_string();
        static_values.urls.insert(url.clone());

        static_values.listeners.retain(|listener| {
            let ret = listener.try_send(Ok(ServiceChange::Insert(url.clone(), ())));
            ret.is_ok()
        });

        Ok(())
    }

    async fn unregister(&self, url: Url) -> Result<(), StdError> {
        let service_name = url.get_service_name();
        let mut lock = self.urls.lock().await;

        match lock.get_mut(&service_name) {
            None => Ok(()),
            Some(static_values) => {
                let url = url.to_string();
                static_values.urls.remove(&url);
                static_values.listeners.retain(|listener| {
                    let ret = listener.try_send(Ok(ServiceChange::Remove(url.clone())));
                    ret.is_ok()
                });
                if static_values.urls.is_empty() {
                    lock.remove(&service_name);
                }
                Ok(())
            }
        }
    }

    async fn subscribe(&self, url: Url) -> Result<DiscoverStream, StdError> {
        let service_name = url.get_service_name();

        let change_rx = {
            let mut lock = self.urls.lock().await;
            let static_values = lock
                .entry(service_name)
                .or_insert_with(|| StaticServiceValues {
                    listeners: Vec::new(),
                    urls: HashSet::new(),
                });

            let (tx, change_rx) = mpsc::channel(64);
            static_values.listeners.push(tx);

            for url in static_values.urls.iter() {
                static_values.listeners.retain(|listener| {
                    let ret = listener.try_send(Ok(ServiceChange::Insert(url.clone(), ())));
                    ret.is_ok()
                });
            }
            change_rx
        };

        Ok(change_rx)
    }

    async fn unsubscribe(&self, url: Url) -> Result<(), StdError> {
        Ok(())
    }
}

#[derive(Error, Debug)]
#[error("static registry error: {0}")]
struct StaticRegistryError(String);

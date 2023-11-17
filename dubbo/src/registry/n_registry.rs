use std::sync::Arc;

use async_trait::async_trait;
use dubbo_base::Url;
use tokio::sync::mpsc::{Receiver, channel};
use tower::discover::Change;


use crate::{StdError, invoker::NewInvoker};

type DiscoverStream = Receiver<Result<Change<String, NewInvoker>, StdError>>;

#[async_trait]
pub trait Registry {

    async fn register(&self, url: Url) -> Result<(), StdError>;
    
    async fn unregister(&self, url: Url) -> Result<(), StdError>;

    // todo service_name change to url
    async fn subscribe(&self, service_name: String) -> Result<DiscoverStream, StdError>;

    async fn unsubscribe(&self, url: Url) -> Result<(), StdError>;
}

#[derive(Clone)]
pub struct ArcRegistry {
    inner: Arc<dyn Registry + Send + Sync + 'static>
}


pub enum RegistryComponent {
    NacosRegistry,
    ZookeeperRegistry,
    StaticRegistry(StaticRegistry),
}


pub struct StaticRegistry {
    urls: Vec<Url>
}

impl ArcRegistry {

    pub fn new(registry: impl Registry + Send + Sync + 'static) -> Self {
        Self { inner: Arc::new(registry) }
    }
}

#[async_trait]
impl Registry for ArcRegistry {
    
    async fn register(&self, url: Url) -> Result<(), StdError> {
        self.register(url).await
    }

    async fn unregister(&self, url: Url) -> Result<(), StdError> {
        self.unregister(url).await
    }

    async fn subscribe(&self, service_name: String) -> Result<DiscoverStream, StdError> {
        self.subscribe(service_name).await
    }

    async fn unsubscribe(&self, url: Url) -> Result<(), StdError> {
        self.unsubscribe(url).await
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

    async fn subscribe(&self, service_name: String) -> Result<DiscoverStream, StdError> {
        todo!()
    }

    async fn unsubscribe(&self, url: Url) -> Result<(), StdError> {
        todo!()
    }
}


impl StaticRegistry {

    pub fn new(urls: Vec<Url>) -> Self {
        Self {
            urls
        }
    }
}


#[async_trait]
impl Registry for StaticRegistry {
    async fn register(&self, url: Url) -> Result<(), StdError> {
        todo!()
    }

    async fn unregister(&self, url: Url) -> Result<(), StdError> {
        todo!()
    }

    async fn subscribe(&self, service_name: String) -> Result<DiscoverStream, StdError> {
        let (tx, rx) = channel(self.urls.len());
        for url in self.urls.iter() {
            let invoker = NewInvoker::new(url.clone());
            let change = Ok(Change::Insert(service_name.clone(), invoker));
            tx.send(change).await?;
        }      

        Ok(rx)
    }

    async fn unsubscribe(&self, url: Url) -> Result<(), StdError> {
        todo!()
    }
}
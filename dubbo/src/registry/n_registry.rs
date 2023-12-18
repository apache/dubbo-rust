use std::{sync::Arc, collections::{HashMap, HashSet}};

use async_trait::async_trait;
use dubbo_base::Url;
use thiserror::Error;
use tokio::sync::{mpsc::{channel, Receiver}, Mutex, watch::{Sender, self}};
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
    sender: Sender<ServiceChange>,
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
            let static_values = map.entry(service_name).or_insert_with(|| {
                let (tx, _) = watch::channel(ServiceChange::Insert(String::default(), ()));
                StaticServiceValues {
                    sender: tx,
                    urls: HashSet::new(),
                }
            });
            let url = url.to_string();
            static_values.urls.insert(url.clone());
            let _ = static_values.sender.send(ServiceChange::Insert(url, ()));
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

        let static_values = lock.entry(service_name).or_insert_with(|| {
            let (tx, _) = watch::channel(ServiceChange::Insert(String::default(), ()));
            StaticServiceValues {
                sender: tx,
                urls: HashSet::new(),
            }
        });
        let url = url.to_string();
        static_values.urls.insert(url.clone());
        let _ = static_values.sender.send(ServiceChange::Insert(url, ()));

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
                let _ = static_values.sender.send(ServiceChange::Remove(url));
                if static_values.urls.is_empty() {
                    lock.remove(&service_name);
                }
                Ok(())
            }
        }
        
    }

    async fn subscribe(&self, url: Url) -> Result<DiscoverStream, StdError> {
        let service_name = url.get_service_name();

        let mut change_rx = {
            let mut lock = self.urls.lock().await;
            let static_values = lock.entry(service_name).or_insert_with(||{
                let (tx, _) = watch::channel(ServiceChange::Insert(String::default(), ()));
                StaticServiceValues {
                    sender: tx,
                    urls: HashSet::new(),
                }
            });
            for url in static_values.urls.iter() {
                let _ = static_values.sender.send(ServiceChange::Insert(url.clone(), ()));
            }
            static_values.sender.subscribe()
        };

      
        let (tx, rx) = channel(64);


        tokio::spawn(async move {
            
            loop {
                let _ = change_rx.changed().await;
       
                let change = match &*change_rx.borrow_and_update() {
                    ServiceChange::Insert(service_url, _) => {
                        Change::Insert(service_url.clone(), ())
                    },
                    ServiceChange::Remove(service_url) => {
                        Change::Remove(service_url.clone())
                    },
                };
               let ret = tx.send(Ok(change)).await;
               match ret {
                     Ok(_) => {},
                     Err(_) => {
                          break;
                     }
               }
               
            }
        });
        Ok(rx)

    }

    async fn unsubscribe(&self, url: Url) -> Result<(), StdError> {
        Ok(())
    }
}

#[derive(Error, Debug)]
#[error("static registry error: {0}")]
struct StaticRegistryError(String);
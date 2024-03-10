use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    convert::Infallible,
    str::FromStr,
};

use async_trait::async_trait;
use itertools::Itertools;
use thiserror::Error;
use tokio::sync::{
    mpsc::{self},
    Mutex,
};

use dubbo_base::{url::UrlParam, Url};

use crate::{
    extension::{
        registry_extension::{DiscoverStream, InterfaceName, Registry, RegistryUrl, ServiceChange},
        Extension, ExtensionName, ExtensionType,
    },
    StdError,
};

pub struct StaticServiceValues {
    listeners: Vec<mpsc::Sender<Result<ServiceChange, StdError>>>,
    urls: HashSet<String>,
}

pub struct StaticRegistry {
    urls: Mutex<HashMap<String, StaticServiceValues>>,
    self_url: Url,
}

impl StaticRegistry {
    pub fn to_extension_url(static_invoker_urls: Vec<Url>) -> Url {
        let static_invoker_urls: StaticInvokerUrls =
            static_invoker_urls.iter().join(",").parse().unwrap();
        let mut static_registry_extension_loader_url: Url = "extension://0.0.0.0".parse().unwrap();

        static_registry_extension_loader_url.add_query_param(ExtensionType::Registry);
        static_registry_extension_loader_url.add_query_param(ExtensionName::new(Self::name()));
        static_registry_extension_loader_url
            .add_query_param(RegistryUrl::new("static://127.0.0.1".parse().unwrap()));
        static_registry_extension_loader_url.add_query_param(static_invoker_urls);

        static_registry_extension_loader_url
    }
}

impl StaticRegistry {
    pub fn new(url: Url) -> Self {
        let static_urls = url.query::<StaticInvokerUrls>();
        let static_urls = match static_urls {
            None => Vec::default(),
            Some(static_urls) => static_urls.value(),
        };

        let mut map = HashMap::with_capacity(static_urls.len());

        for url in static_urls {
            let service_name = url.query::<InterfaceName>().unwrap();
            let service_name = service_name.value();

            let static_values = map
                .entry(service_name)
                .or_insert_with(|| StaticServiceValues {
                    listeners: Vec::new(),
                    urls: HashSet::new(),
                });
            let url = url.to_string();
            static_values.urls.insert(url.clone());
        }

        let self_url = "static://0.0.0.0".parse().unwrap();

        Self {
            urls: Mutex::new(map),
            self_url,
        }
    }
}

impl Default for StaticRegistry {
    fn default() -> Self {
        let self_url = "static://0.0.0.0".parse().unwrap();

        Self {
            self_url,
            urls: Mutex::new(HashMap::new()),
        }
    }
}
#[async_trait]
impl Registry for StaticRegistry {
    async fn register(&self, url: Url) -> Result<(), StdError> {
        let service_name = url.query::<InterfaceName>().unwrap();
        let service_name = service_name.value();

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
        let service_name = url.query::<InterfaceName>().unwrap();
        let service_name = service_name.value();

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
        let service_name = url.query::<InterfaceName>().unwrap();
        let service_name = service_name.value();

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

            for listener in &static_values.listeners {
                for url in &static_values.urls {
                    let _ = listener
                        .send(Ok(ServiceChange::Insert(url.clone(), ())))
                        .await;
                }
            }

            change_rx
        };

        Ok(change_rx)
    }

    async fn unsubscribe(&self, url: Url) -> Result<(), StdError> {
        Ok(())
    }

    fn url(&self) -> &Url {
        &self.self_url
    }
}

#[async_trait::async_trait]
impl Extension for StaticRegistry {
    type Target = Box<dyn Registry + Send + 'static>;

    fn name() -> String {
        "static".to_string()
    }

    async fn create(url: &Url) -> Result<Self::Target, StdError> {
        // url example:
        // extension://0.0.0.0?extension-type=registry&extension-name=static&registry=static://127.0.0.1
        let static_invoker_urls = url.query::<StaticInvokerUrls>();

        let registry_url = url.query::<RegistryUrl>().unwrap();
        let mut registry_url = registry_url.value();

        if let Some(static_invoker_urls) = static_invoker_urls {
            registry_url.add_query_param(static_invoker_urls);
        }

        let static_registry = StaticRegistry::new(registry_url);

        Ok(Box::new(static_registry))
    }
}
#[derive(Error, Debug)]
#[error("static registry error: {0}")]
struct StaticRegistryError(String);

pub(crate) struct StaticInvokerUrls(String);

impl UrlParam for StaticInvokerUrls {
    type TargetType = Vec<Url>;

    fn name() -> &'static str {
        "static-invoker-urls"
    }

    fn value(&self) -> Self::TargetType {
        self.0.split(",").map(|url| url.parse().unwrap()).collect()
    }

    fn as_str(&self) -> Cow<str> {
        Cow::Borrowed(&self.0)
    }
}

impl FromStr for StaticInvokerUrls {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl Default for StaticInvokerUrls {
    fn default() -> Self {
        Self(String::default())
    }
}

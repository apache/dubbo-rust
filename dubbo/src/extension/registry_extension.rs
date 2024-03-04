use crate::extension::{ExtensionLoaderName, ExtensionName, ExtensionType};
use crate::StdError;
use dubbo_base::url::UrlParam;
use dubbo_base::Url;
use std::borrow::Cow;
use std::convert::Infallible;
use std::str::FromStr;

pub fn to_extension_url(registry_url: Url) -> Url {
    let mut registry_extension_loader_url: Url = "extension://127.0.0.1".parse().unwrap();

    let protocol = registry_url.protocol();

    registry_extension_loader_url.add_query_param(ExtensionType::Registry);
    registry_extension_loader_url.add_query_param(ExtensionLoaderName::new(protocol));
    registry_extension_loader_url.add_query_param(ExtensionName::new(
        registry_url.short_url_without_query().as_str(),
    ));
    registry_extension_loader_url.add_query_param(RegistryUrl::new(registry_url));

    registry_extension_loader_url
}

pub mod proxy {
    use crate::registry::n_registry::{DiscoverStream, Registry};
    use crate::StdError;
    use async_trait::async_trait;
    use dubbo_base::Url;
    use dubbo_logger::tracing::error;
    use thiserror::Error;
    use tokio::sync::oneshot;

    pub enum RegistryOpt {
        Register(Url, oneshot::Sender<Result<(), StdError>>),
        Unregister(Url, oneshot::Sender<Result<(), StdError>>),
        Subscribe(Url, oneshot::Sender<Result<DiscoverStream, StdError>>),
        UnSubscribe(Url, oneshot::Sender<Result<(), StdError>>),
    }

    #[derive(Clone)]
    pub struct RegistryProxy {
        sender: tokio::sync::mpsc::Sender<RegistryOpt>,
        url: Url,
    }

    #[async_trait]
    impl Registry for RegistryProxy {
        async fn register(&self, url: Url) -> Result<(), StdError> {
            let (tx, rx) = oneshot::channel();

            match self
                .sender
                .send(RegistryOpt::Register(url.clone(), tx))
                .await
            {
                Ok(_) => match rx.await {
                    Ok(result) => result,
                    Err(_) => {
                        error!(
                            "registry proxy error: receive register response failed, url: {}",
                            url
                        );
                        return Err(
                            RegistryProxyError::new("receive register response failed").into()
                        );
                    }
                },
                Err(_) => {
                    error!(
                        "registry proxy error: send register request failed, url: {}",
                        url
                    );
                    return Err(RegistryProxyError::new("send register opt failed").into());
                }
            }
        }

        async fn unregister(&self, url: Url) -> Result<(), StdError> {
            let (tx, rx) = oneshot::channel();
            match self
                .sender
                .send(RegistryOpt::Unregister(url.clone(), tx))
                .await
            {
                Ok(_) => match rx.await {
                    Ok(result) => result,
                    Err(_) => {
                        error!(
                            "registry proxy error: receive unregister response failed, url: {}",
                            url
                        );
                        return Err(
                            RegistryProxyError::new("receive unregister response failed").into(),
                        );
                    }
                },
                Err(_) => {
                    error!(
                        "registry proxy error: send unregister request failed, url: {}",
                        url
                    );
                    return Err(RegistryProxyError::new("send unregister opt failed").into());
                }
            }
        }

        async fn subscribe(&self, url: Url) -> Result<DiscoverStream, StdError> {
            let (tx, rx) = oneshot::channel();

            match self
                .sender
                .send(RegistryOpt::Subscribe(url.clone(), tx))
                .await
            {
                Ok(_) => match rx.await {
                    Ok(result) => result,
                    Err(_) => {
                        error!(
                            "registry proxy error: receive subscribe response failed, url: {}",
                            url
                        );
                        return Err(
                            RegistryProxyError::new("receive subscribe response failed").into()
                        );
                    }
                },
                Err(_) => {
                    error!(
                        "registry proxy error: send subscribe request failed, url: {}",
                        url
                    );
                    return Err(RegistryProxyError::new("send subscribe opt failed").into());
                }
            }
        }

        async fn unsubscribe(&self, url: Url) -> Result<(), StdError> {
            let (tx, rx) = oneshot::channel();
            match self
                .sender
                .send(RegistryOpt::UnSubscribe(url.clone(), tx))
                .await
            {
                Ok(_) => {
                    match rx.await {
                        Ok(result) => result,
                        Err(_) => {
                            error!("registry proxy error: receive unsubscribe response failed, url: {}", url);
                            return Err(RegistryProxyError::new(
                                "receive unsubscribe response failed",
                            )
                            .into());
                        }
                    }
                }
                Err(_) => {
                    error!(
                        "registry proxy error: send unsubscribe request failed, url: {}",
                        url
                    );
                    return Err(RegistryProxyError::new("send unsubscribe opt failed").into());
                }
            }
        }

        fn url(&self) -> &Url {
            &self.url
        }
    }

    impl From<Box<dyn Registry + Send>> for RegistryProxy {
        fn from(registry: Box<dyn Registry + Send>) -> Self {
            let url = registry.url().clone();

            let (sender, mut receiver) = tokio::sync::mpsc::channel(1024);

            tokio::spawn(async move {
                while let Some(opt) = receiver.recv().await {
                    match opt {
                        RegistryOpt::Register(url, tx) => {
                            let register = registry.register(url).await;
                            if let Err(_) = tx.send(register) {
                                error!("registry proxy error: send register response failed");
                            }
                        }
                        RegistryOpt::Unregister(url, tx) => {
                            let unregister = registry.unregister(url).await;
                            if let Err(_) = tx.send(unregister) {
                                error!("registry proxy error: send unregister response failed");
                            }
                        }
                        RegistryOpt::Subscribe(url, tx) => {
                            let subscribe = registry.subscribe(url).await;
                            if let Err(_) = tx.send(subscribe) {
                                error!("registry proxy error: send subscribe response failed");
                            }
                        }
                        RegistryOpt::UnSubscribe(url, tx) => {
                            let unsubscribe = registry.unsubscribe(url).await;
                            if let Err(_) = tx.send(unsubscribe) {
                                error!("registry proxy error: send unsubscribe response failed");
                            }
                        }
                    }
                }
            });

            RegistryProxy { sender, url }
        }
    }

    #[derive(Error, Debug)]
    #[error("registry proxy error: {0}")]
    pub(crate) struct RegistryProxyError(String);

    impl RegistryProxyError {
        pub(crate) fn new(msg: &str) -> Self {
            RegistryProxyError(msg.to_string())
        }
    }
}

pub struct RegistryUrl(Url);

impl RegistryUrl {
    pub fn new(url: Url) -> Self {
        Self(url)
    }
}

impl UrlParam for RegistryUrl {
    type TargetType = Url;

    fn name() -> &'static str {
        "registry"
    }

    fn value(&self) -> Self::TargetType {
        self.0.clone()
    }

    fn as_str<'a>(&'a self) -> std::borrow::Cow<'a, str> {
        self.0.as_str().into()
    }
}

impl FromStr for RegistryUrl {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

pub struct ServiceNamespace(String);

impl ServiceNamespace {
    pub fn new(namespace: String) -> Self {
        Self(namespace)
    }
}

impl UrlParam for ServiceNamespace {
    type TargetType = String;

    fn name() -> &'static str {
        "namespace"
    }

    fn value(&self) -> Self::TargetType {
        self.0.clone()
    }

    fn as_str<'a>(&'a self) -> std::borrow::Cow<'a, str> {
        self.0.as_str().into()
    }
}

impl FromStr for ServiceNamespace {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl Default for ServiceNamespace {
    fn default() -> Self {
        Self("public".to_string())
    }
}

pub struct AppName(String);

impl AppName {
    pub fn new(app_name: String) -> Self {
        Self(app_name)
    }
}

impl UrlParam for AppName {
    type TargetType = String;

    fn name() -> &'static str {
        "app_name"
    }

    fn value(&self) -> Self::TargetType {
        self.0.clone()
    }

    fn as_str<'a>(&'a self) -> std::borrow::Cow<'a, str> {
        self.0.as_str().into()
    }
}

impl FromStr for AppName {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl Default for AppName {
    fn default() -> Self {
        Self("UnknownApp".to_string())
    }
}

pub struct InterfaceName(String);

impl InterfaceName {
    pub fn new(interface_name: String) -> Self {
        Self(interface_name)
    }
}

impl UrlParam for InterfaceName {
    type TargetType = String;

    fn name() -> &'static str {
        "interface"
    }

    fn value(&self) -> Self::TargetType {
        self.0.clone()
    }

    fn as_str<'a>(&'a self) -> std::borrow::Cow<'a, str> {
        self.0.as_str().into()
    }
}

impl FromStr for InterfaceName {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl Default for InterfaceName {
    fn default() -> Self {
        Self("".to_string())
    }
}

pub struct Category(String);

impl Category {
    pub fn new(category: String) -> Self {
        Self(category)
    }
}

impl UrlParam for Category {
    type TargetType = String;

    fn name() -> &'static str {
        "category"
    }

    fn value(&self) -> Self::TargetType {
        self.0.clone()
    }

    fn as_str<'a>(&'a self) -> std::borrow::Cow<'a, str> {
        self.0.as_str().into()
    }
}

impl FromStr for Category {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl Default for Category {
    fn default() -> Self {
        Self("".to_string())
    }
}

pub struct Version(String);

impl Version {
    pub fn new(version: String) -> Self {
        Self(version)
    }
}

impl UrlParam for Version {
    type TargetType = String;

    fn name() -> &'static str {
        "version"
    }

    fn value(&self) -> Self::TargetType {
        self.0.clone()
    }

    fn as_str<'a>(&'a self) -> std::borrow::Cow<'a, str> {
        self.0.as_str().into()
    }
}

impl FromStr for Version {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl Default for Version {
    fn default() -> Self {
        Self("".to_string())
    }
}

pub struct Group(String);

impl Group {
    pub fn new(group: String) -> Self {
        Self(group)
    }
}

impl UrlParam for Group {
    type TargetType = String;

    fn name() -> &'static str {
        "group"
    }

    fn value(&self) -> Self::TargetType {
        self.0.clone()
    }

    fn as_str<'a>(&'a self) -> std::borrow::Cow<'a, str> {
        self.0.as_str().into()
    }
}

impl FromStr for Group {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl Default for Group {
    fn default() -> Self {
        Self("".to_string())
    }
}

pub enum Side {
    Provider,
    Consumer,
}

impl UrlParam for Side {
    type TargetType = String;

    fn name() -> &'static str {
        "side"
    }

    fn value(&self) -> Self::TargetType {
        match self {
            Side::Consumer => "consumer".to_owned(),
            Side::Provider => "provider".to_owned(),
        }
    }

    fn as_str<'a>(&'a self) -> Cow<'a, str> {
        match self {
            Side::Consumer => Cow::Borrowed("consumer"),
            Side::Provider => Cow::Borrowed("provider"),
        }
    }
}

impl FromStr for Side {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "consumer" => Ok(Side::Consumer),
            "provider" => Ok(Side::Provider),
            _ => Ok(Side::Consumer),
        }
    }
}

impl Default for Side {
    fn default() -> Self {
        Side::Consumer
    }
}

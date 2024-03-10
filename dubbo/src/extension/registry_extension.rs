use std::{
    borrow::Cow, collections::HashMap, convert::Infallible, future::Future, pin::Pin, str::FromStr,
};

use async_trait::async_trait;
use thiserror::Error;
use tokio::sync::mpsc::Receiver;
use tower::discover::Change;

use dubbo_base::{url::UrlParam, Url};
use proxy::RegistryProxy;

use crate::{
    extension::{
        ConvertToExtensionFactories, Extension, ExtensionFactories, ExtensionMetaInfo,
        ExtensionName, ExtensionType,
    },
    StdError,
};

// extension://0.0.0.0/?extension-type=registry&extension-name=nacos&registry-url=nacos://127.0.0.1:8848
pub fn to_extension_url(registry_url: Url) -> Url {
    let mut registry_extension_loader_url: Url = "extension://0.0.0.0".parse().unwrap();

    let protocol = registry_url.protocol();

    registry_extension_loader_url.add_query_param(ExtensionType::Registry);
    registry_extension_loader_url.add_query_param(ExtensionName::new(protocol.to_string()));
    registry_extension_loader_url.add_query_param(RegistryUrl::new(registry_url));

    registry_extension_loader_url
}

pub type ServiceChange = Change<String, ()>;
pub type DiscoverStream = Receiver<Result<ServiceChange, StdError>>;

#[async_trait]
pub trait Registry {
    async fn register(&self, url: Url) -> Result<(), StdError>;

    async fn unregister(&self, url: Url) -> Result<(), StdError>;

    async fn subscribe(&self, url: Url) -> Result<DiscoverStream, StdError>;

    async fn unsubscribe(&self, url: Url) -> Result<(), StdError>;

    fn url(&self) -> &Url;
}

impl<T> crate::extension::Sealed for T where T: Registry + Send + 'static {}

impl<T> ExtensionMetaInfo for T
where
    T: Registry + Send + 'static,
    T: Extension<Target = Box<dyn Registry + Send + 'static>>,
{
    fn extension_type() -> ExtensionType {
        ExtensionType::Registry
    }
}

impl<T> ConvertToExtensionFactories for T
where
    T: Registry + Send + 'static,
    T: Extension<Target = Box<dyn Registry + Send + 'static>>,
{
    fn convert_to_extension_factories() -> ExtensionFactories {
        fn constrain<F>(f: F) -> F
        where
            F: for<'a> Fn(
                &'a Url,
            ) -> Pin<
                Box<
                    dyn Future<Output = Result<Box<dyn Registry + Send + 'static>, StdError>>
                        + Send
                        + 'a,
                >,
            >,
        {
            f
        }

        let constructor = constrain(|url: &Url| {
            let f = <T as Extension>::create(url);
            Box::pin(f)
        });

        ExtensionFactories::RegistryExtensionFactory(RegistryExtensionFactory::new(constructor))
    }
}

#[derive(Default)]
pub(super) struct RegistryExtensionLoader {
    factories: HashMap<String, RegistryExtensionFactory>,
}

impl RegistryExtensionLoader {
    pub(super) fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    pub(crate) async fn register(
        &mut self,
        extension_name: String,
        factory: RegistryExtensionFactory,
    ) {
        self.factories.insert(extension_name, factory);
    }

    pub(crate) async fn remove(&mut self, extension_name: String) {
        self.factories.remove(&extension_name);
    }

    pub(crate) async fn load(&mut self, url: &Url) -> Result<RegistryProxy, StdError> {
        let extension_name = url.query::<ExtensionName>().unwrap();
        let extension_name = extension_name.value();
        let factory = self.factories.get_mut(&extension_name).ok_or_else(|| {
            RegistryExtensionLoaderError::new(format!(
                "registry extension loader error: extension name {} not found",
                extension_name
            ))
        })?;
        factory.create(url).await
    }
}

type RegistryConstructor = for<'a> fn(
    &'a Url,
) -> Pin<
    Box<dyn Future<Output = Result<Box<dyn Registry + Send + 'static>, StdError>> + Send + 'a>,
>;
pub(super) struct RegistryExtensionFactory {
    constructor: RegistryConstructor,
    instances: HashMap<String, RegistryProxy>,
}

impl RegistryExtensionFactory {
    pub(super) fn new(constructor: RegistryConstructor) -> Self {
        Self {
            constructor,
            instances: HashMap::new(),
        }
    }
}

impl RegistryExtensionFactory {
    pub(super) async fn create(&mut self, url: &Url) -> Result<RegistryProxy, StdError> {
        let registry_url = url.query::<RegistryUrl>().unwrap();
        let registry_url = registry_url.value();
        let url_str = registry_url.as_str().to_string();
        match self.instances.get(&url_str) {
            Some(proxy) => {
                let proxy = proxy.clone();
                Ok(proxy)
            }
            None => {
                let registry = (self.constructor)(url).await?;
                let proxy = <RegistryProxy as From<Box<dyn Registry + Send>>>::from(registry);
                self.instances.insert(url_str, proxy.clone());
                Ok(proxy)
            }
        }
    }
}

#[derive(Error, Debug)]
#[error("{0}")]
pub(crate) struct RegistryExtensionLoaderError(String);

impl RegistryExtensionLoaderError {
    pub(crate) fn new(msg: String) -> Self {
        RegistryExtensionLoaderError(msg)
    }
}

pub mod proxy {
    use async_trait::async_trait;
    use thiserror::Error;
    use tokio::sync::oneshot;

    
    use dubbo_base::Url;
    use dubbo_logger::tracing::error;

    use crate::{
        extension::registry_extension::{DiscoverStream, Registry},
        StdError,
    };

    pub(super) enum RegistryOpt {
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

    fn as_str<'a>(&'a self) -> Cow<'a, str> {
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

    fn as_str<'a>(&'a self) -> Cow<'a, str> {
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

    fn as_str<'a>(&'a self) -> Cow<'a, str> {
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

    fn as_str<'a>(&'a self) -> Cow<'a, str> {
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

    fn as_str<'a>(&'a self) -> Cow<'a, str> {
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

    fn as_str<'a>(&'a self) -> Cow<'a, str> {
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

    fn as_str<'a>(&'a self) -> Cow<'a, str> {
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

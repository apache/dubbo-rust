pub mod registry_extension;

use crate::extension::registry_extension::proxy::RegistryProxy;
use crate::extension::registry_extension::RegistryUrl;
use crate::registry::n_registry::{Registry, StaticInvokerUrls, StaticRegistryExtensionLoader};
use crate::StdError;
use dubbo_base::url::UrlParam;
use dubbo_base::Url;
use dubbo_logger::tracing::{debug, error};
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::Infallible;
use std::str::FromStr;
use thiserror::Error;
use tokio::sync::oneshot;

pub static INSTANCE: once_cell::sync::Lazy<ExtensionDirectoryCommander> =
    once_cell::sync::Lazy::new(|| ExtensionDirectory::init());

#[derive(Default)]
struct ExtensionDirectory {
    registry_extension_loaders: HashMap<String, RegistryExtensionLoaderWrapper>,
}

impl ExtensionDirectory {
    fn init() -> ExtensionDirectoryCommander {
        let (tx, rx) = tokio::sync::mpsc::channel::<ExtensionOpt>(64);

        let mut directory = ExtensionDirectory::default();

        let static_registry_extension_loader = StaticRegistryExtensionLoader;

        directory.add_registry_extension_loader(Box::new(static_registry_extension_loader));

        tokio::spawn(async move {
            directory.run(rx).await;
        });

        ExtensionDirectoryCommander::new(tx)
    }

    async fn run(mut self, mut rx: tokio::sync::mpsc::Receiver<ExtensionOpt>) {
        while let Some(opt) = rx.recv().await {
            match opt {
                ExtensionOpt::AddRegistryExtensionLoader(loader) => {
                    self.add_registry_extension_loader(loader)
                }

                ExtensionOpt::RemoveRegistryExtensionLoader(name) => {
                    self.remove_registry_extension_loader(&name)
                }

                ExtensionOpt::LoadRegistryExtension(url, tx) => {
                    self.load_registry_extension(url, tx).await
                }
            }
        }
    }

    fn add_registry_extension_loader(&mut self, loader: Box<dyn RegistryExtensionLoader + Send>) {
        let name = loader.name();
        debug!("add registry extension loader, name: {}", name);
        self.registry_extension_loaders
            .insert(name, RegistryExtensionLoaderWrapper::new(loader));
    }

    fn remove_registry_extension_loader(&mut self, name: &str) {
        debug!("remove registry extension loader, name: {}", name);
        self.registry_extension_loaders.remove(name);
    }

    async fn load_registry_extension(&mut self, url: Url, tx: oneshot::Sender<RegistryProxy>) {
        let extension_loader_name = url.query::<ExtensionLoaderName>();
        match extension_loader_name {
            Some(extension_loader_name) => {
                let extension_loader_name = extension_loader_name.value();
                if let Some(loader) = self
                    .registry_extension_loaders
                    .get_mut(&extension_loader_name)
                {
                    match loader.load(&url).await {
                        Ok(extension) => match tx.send(extension) {
                            Ok(_) => {}
                            Err(_) => {
                                error!("load registry extension error: send load extension response failed, url: {}", url);
                            }
                        },
                        Err(err) => {
                            error!("load registry extension error: {}", err);
                        }
                    }
                } else {
                    error!(
                        "load registry extension error: extension loader not found, name: {}",
                        extension_loader_name
                    );
                }
            }
            None => {
                error!(
                    "load registry extension error: extension name not found, url: {}",
                    url
                );
            }
        }
    }
}

pub struct ExtensionDirectoryCommander {
    sender: tokio::sync::mpsc::Sender<ExtensionOpt>,
}

impl ExtensionDirectoryCommander {
    fn new(sender: tokio::sync::mpsc::Sender<ExtensionOpt>) -> Self {
        ExtensionDirectoryCommander { sender }
    }

    pub async fn add_registry_extension_loader(
        &self,
        loader: Box<dyn RegistryExtensionLoader + Send>,
    ) -> Result<(), StdError> {
        match self
            .sender
            .send(ExtensionOpt::AddRegistryExtensionLoader(loader))
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => {
                Err(AddExtensionLoaderError::new("add registry extension loader failed").into())
            }
        }
    }

    pub async fn remove_registry_extension_loader(&self, name: &str) -> Result<(), StdError> {
        match self
            .sender
            .send(ExtensionOpt::RemoveRegistryExtensionLoader(
                name.to_string(),
            ))
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(RemoveExtensionLoaderError::new(
                "remove registry extension loader failed",
            )
            .into()),
        }
    }

    pub async fn load_registry_extension(&self, url: Url) -> Result<RegistryProxy, StdError> {
        let (tx, rx) = oneshot::channel();
        match self
            .sender
            .send(ExtensionOpt::LoadRegistryExtension(url.clone(), tx))
            .await
        {
            Ok(_) => match rx.await {
                Ok(result) => Ok(result),
                Err(_) => {
                    error!("load registry extension error: receive load extension response failed, url: {}", url);
                    return Err(
                        LoadExtensionError::new("receive load extension response failed").into(),
                    );
                }
            },
            Err(_) => {
                error!(
                    "load registry extension error: send load extension request failed, url: {}",
                    url
                );
                return Err(LoadExtensionError::new("send load extension request failed").into());
            }
        }
    }
}

enum ExtensionOpt {
    AddRegistryExtensionLoader(Box<dyn RegistryExtensionLoader + Send>),

    RemoveRegistryExtensionLoader(String),

    LoadRegistryExtension(Url, oneshot::Sender<RegistryProxy>),
}

#[derive(Error, Debug)]
#[error("{0}")]
pub struct AddExtensionLoaderError(String);

impl AddExtensionLoaderError {
    pub fn new(msg: &str) -> Self {
        AddExtensionLoaderError(msg.to_string())
    }
}

#[derive(Error, Debug)]
#[error("{0}")]
pub struct RemoveExtensionLoaderError(String);

impl RemoveExtensionLoaderError {
    pub fn new(msg: &str) -> Self {
        RemoveExtensionLoaderError(msg.to_string())
    }
}

#[derive(Error, Debug)]
#[error("{0}")]
pub struct LoadExtensionError(String);

impl LoadExtensionError {
    pub fn new(msg: &str) -> Self {
        LoadExtensionError(msg.to_string())
    }
}

macro_rules! extension_loader {
    ($name:ident<$extension_proxy_type:tt>) => {
        #[async_trait::async_trait]
        pub trait $name {
            fn name(&self) -> String;

            async fn load(&mut self, url: &Url) -> Result<$extension_proxy_type, StdError>;
        }
    };
}

extension_loader!(RegistryExtensionLoader<RegistryProxy>);

macro_rules! extension_loader_wrapper {
    ($loader_wrapper:ident[$loader:ident<$extension_type:tt<=>$proxy_type:tt>]) => {
        extension_loader_wrapper!($loader_wrapper, $loader, $extension_type, $proxy_type);
    };
    ($loader_wrapper:ident, $loader:ident, $extension_type:tt, $proxy_type:tt) => {
        struct $loader_wrapper {
            loader: Box<dyn $loader + Send>,
            extensions: HashMap<String, $proxy_type>,
        }

        impl $loader_wrapper {
            fn new(loader: Box<dyn $loader + Send>) -> Self {
                $loader_wrapper {
                    loader,
                    extensions: HashMap::new(),
                }
            }
        }

        #[async_trait::async_trait]
        impl $loader for $loader_wrapper {
            fn name(&self) -> String {
                self.loader.name()
            }

            async fn load(&mut self, url: &Url) -> Result<$proxy_type, StdError> {
                let extension_name = url.query::<ExtensionName>();
                let extension_name = match extension_name {
                    None => "default".to_string(),
                    Some(extension_name) => extension_name.value(),
                };

                if let Some(extension_proxy) = self.extensions.get(&extension_name) {
                    return Ok(extension_proxy.clone());
                }

                let extension = self.loader.load(url).await?;
                let extension = $proxy_type::from(extension);
                self.extensions.insert(extension_name, extension.clone());
                Ok(extension)
            }
        }
    };
}

extension_loader_wrapper!(RegistryExtensionLoaderWrapper[RegistryExtensionLoader<Registry<=>RegistryProxy>]);

pub struct ExtensionLoaderName(String);

impl ExtensionLoaderName {
    pub fn new(name: &str) -> Self {
        ExtensionLoaderName(name.to_string())
    }
}

impl UrlParam for ExtensionLoaderName {
    type TargetType = String;

    fn name() -> &'static str {
        "extension-loader-name"
    }

    fn value(&self) -> Self::TargetType {
        self.0.clone()
    }

    fn as_str<'a>(&'a self) -> std::borrow::Cow<'a, str> {
        self.0.as_str().into()
    }
}

impl FromStr for ExtensionLoaderName {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ExtensionLoaderName::new(s))
    }
}

pub struct ExtensionName(String);

impl ExtensionName {
    pub fn new(name: &str) -> Self {
        ExtensionName(name.to_string())
    }
}

impl UrlParam for ExtensionName {
    type TargetType = String;

    fn name() -> &'static str {
        "extension-name"
    }

    fn value(&self) -> Self::TargetType {
        self.0.clone()
    }

    fn as_str<'a>(&'a self) -> std::borrow::Cow<'a, str> {
        self.0.as_str().into()
    }
}

impl FromStr for ExtensionName {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ExtensionName::new(s))
    }
}

pub enum ExtensionType {
    Registry,
}

impl UrlParam for ExtensionType {
    type TargetType = String;

    fn name() -> &'static str {
        "extension-type"
    }

    fn value(&self) -> Self::TargetType {
        match self {
            ExtensionType::Registry => "registry".to_owned(),
        }
    }

    fn as_str<'a>(&'a self) -> Cow<'a, str> {
        match self {
            ExtensionType::Registry => Cow::Borrowed("registry"),
        }
    }
}

impl FromStr for ExtensionType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "registry" => Ok(ExtensionType::Registry),
            _ => panic!("the extension type enum is not in range"),
        }
    }
}

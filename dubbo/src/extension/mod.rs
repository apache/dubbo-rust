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

mod invoker_extension;
pub mod registry_extension;

use crate::{
    extension::{
        invoker_extension::proxy::InvokerProxy,
        registry_extension::{proxy::RegistryProxy, RegistryExtension},
    },
    logger::tracing::{error, info},
    params::extension_param::ExtensionType,
    registry::registry::StaticRegistry,
    url::UrlParam,
    StdError, Url,
};
use std::{future::Future, pin::Pin, sync::Arc};
use thiserror::Error;
use tokio::sync::{oneshot, RwLock};

pub static EXTENSIONS: once_cell::sync::Lazy<ExtensionDirectoryCommander> =
    once_cell::sync::Lazy::new(|| ExtensionDirectory::init());

#[derive(Default)]
struct ExtensionDirectory {
    registry_extension_loader: registry_extension::RegistryExtensionLoader,
    invoker_extension_loader: invoker_extension::InvokerExtensionLoader,
}

impl ExtensionDirectory {
    fn init() -> ExtensionDirectoryCommander {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<ExtensionOpt>(64);

        tokio::spawn(async move {
            let mut extension_directory = ExtensionDirectory::default();

            // register static registry extension
            let _ = extension_directory.register(
                StaticRegistry::name(),
                RegistryExtension::<StaticRegistry>::extension_factory(),
                RegistryExtension::<StaticRegistry>::extension_type(),
            );

            while let Some(extension_opt) = rx.recv().await {
                match extension_opt {
                    ExtensionOpt::Register(
                        extension_name,
                        extension_factories,
                        extension_type,
                        tx,
                    ) => {
                        let result = extension_directory.register(
                            extension_name,
                            extension_factories,
                            extension_type,
                        );
                        let _ = tx.send(result);
                    }
                    ExtensionOpt::Remove(extension_name, extension_type, tx) => {
                        let result = extension_directory.remove(extension_name, extension_type);
                        let _ = tx.send(result);
                    }
                    ExtensionOpt::Load(url, extension_type, tx) => {
                        let _ = extension_directory.load(url, extension_type, tx);
                    }
                }
            }
        });

        ExtensionDirectoryCommander { sender: tx }
    }

    fn register(
        &mut self,
        extension_name: String,
        extension_factories: ExtensionFactories,
        extension_type: ExtensionType,
    ) -> Result<(), StdError> {
        match extension_type {
            ExtensionType::Registry => match extension_factories {
                ExtensionFactories::RegistryExtensionFactory(registry_extension_factory) => {
                    self.registry_extension_loader
                        .register(extension_name, registry_extension_factory);
                    Ok(())
                }
                _ => Ok(()),
            },
            ExtensionType::Invoker => match extension_factories {
                ExtensionFactories::InvokerExtensionFactory(invoker_extension_factory) => {
                    self.invoker_extension_loader
                        .register(extension_name, invoker_extension_factory);
                    Ok(())
                }
                _ => Ok(()),
            },
        }
    }

    fn remove(
        &mut self,
        extension_name: String,
        extension_type: ExtensionType,
    ) -> Result<(), StdError> {
        match extension_type {
            ExtensionType::Registry => {
                self.registry_extension_loader.remove(extension_name);
                Ok(())
            }
            ExtensionType::Invoker => {
                self.invoker_extension_loader.remove(extension_name);
                Ok(())
            }
        }
    }

    fn load(
        &mut self,
        url: Url,
        extension_type: ExtensionType,
        callback: oneshot::Sender<Result<Extensions, StdError>>,
    ) {
        match extension_type {
            ExtensionType::Registry => {
                let extension = self.registry_extension_loader.load(url);
                match extension {
                    Ok(mut extension) => {
                        tokio::spawn(async move {
                            let extension = extension.resolve().await;
                            match extension {
                                Ok(extension) => {
                                    let _ = callback.send(Ok(Extensions::Registry(extension)));
                                }
                                Err(err) => {
                                    error!("load registry extension failed: {}", err);
                                    let _ = callback.send(Err(err));
                                }
                            }
                        });
                    }
                    Err(err) => {
                        error!("load registry extension failed: {}", err);
                        let _ = callback.send(Err(err));
                    }
                }
            }
            ExtensionType::Invoker => {
                let extension = self.invoker_extension_loader.load(url);
                match extension {
                    Ok(mut extension) => {
                        tokio::spawn(async move {
                            let invoker = extension.resolve().await;
                            match invoker {
                                Ok(invoker) => {
                                    let _ = callback.send(Ok(Extensions::Invoker(invoker)));
                                }
                                Err(err) => {
                                    error!("load invoker extension failed: {}", err);
                                    let _ = callback.send(Err(err));
                                }
                            }
                        });
                    }
                    Err(err) => {
                        error!("load invoker extension failed: {}", err);
                        let _ = callback.send(Err(err));
                    }
                }
            }
        }
    }
}

type ExtensionCreator<T> = Box<
    dyn Fn(Url) -> Pin<Box<dyn Future<Output = Result<T, StdError>> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;
pub(crate) struct ExtensionPromiseResolver<T> {
    resolved_data: Option<T>,
    creator: ExtensionCreator<T>,
    url: Url,
}

impl<T> ExtensionPromiseResolver<T>
where
    T: Send + Clone + 'static,
{
    fn new(creator: ExtensionCreator<T>, url: Url) -> Self {
        ExtensionPromiseResolver {
            resolved_data: None,
            creator,
            url,
        }
    }

    fn resolved_data(&self) -> Option<T> {
        self.resolved_data.clone()
    }

    async fn resolve(&mut self) -> Result<T, StdError> {
        match (self.creator)(self.url.clone()).await {
            Ok(data) => {
                self.resolved_data = Some(data.clone());
                Ok(data)
            }
            Err(err) => {
                error!("create extension failed: {}", err);
                Err(LoadExtensionError::new(
                    "load extension failed, create extension occur an error".to_string(),
                )
                .into())
            }
        }
    }
}

pub(crate) struct LoadExtensionPromise<T> {
    resolver: Arc<RwLock<ExtensionPromiseResolver<T>>>,
}

impl<T> LoadExtensionPromise<T>
where
    T: Send + Clone + 'static,
{
    pub(crate) fn new(creator: ExtensionCreator<T>, url: Url) -> Self {
        let resolver = ExtensionPromiseResolver::new(creator, url);
        LoadExtensionPromise {
            resolver: Arc::new(RwLock::new(resolver)),
        }
    }

    pub(crate) async fn resolve(&mut self) -> Result<T, StdError> {
        // get read lock
        let resolver_read_lock = self.resolver.read().await;
        // if extension is not None, return it
        if let Some(extension) = resolver_read_lock.resolved_data() {
            return Ok(extension);
        }
        drop(resolver_read_lock);

        let mut write_lock = self.resolver.write().await;

        match write_lock.resolved_data() {
            Some(extension) => Ok(extension),
            None => {
                let extension = write_lock.resolve().await;
                extension
            }
        }
    }
}

impl<T> Clone for LoadExtensionPromise<T> {
    fn clone(&self) -> Self {
        LoadExtensionPromise {
            resolver: self.resolver.clone(),
        }
    }
}

pub struct ExtensionDirectoryCommander {
    sender: tokio::sync::mpsc::Sender<ExtensionOpt>,
}

impl ExtensionDirectoryCommander {
    #[allow(private_bounds)]
    pub async fn register<T>(&self) -> Result<(), StdError>
    where
        T: ExtensionMetaInfo,
    {
        let extension_name = T::name();
        let extension_factories = T::extension_factory();
        let extension_type = T::extension_type();

        info!(
            "register extension: {}, type: {}",
            extension_name,
            extension_type.as_str()
        );

        let (tx, rx) = oneshot::channel();

        let send = self
            .sender
            .send(ExtensionOpt::Register(
                extension_name.clone(),
                extension_factories,
                extension_type,
                tx,
            ))
            .await;

        let Ok(_) = send else {
            let err_msg = format!("register extension {} failed", extension_name);
            return Err(RegisterExtensionError::new(err_msg).into());
        };

        let ret = rx.await;

        match ret {
            Ok(_) => Ok(()),
            Err(_) => {
                let err_msg = format!("register extension {} failed", extension_name);
                Err(RegisterExtensionError::new(err_msg).into())
            }
        }
    }

    #[allow(private_bounds)]
    pub async fn remove<T>(&self) -> Result<(), StdError>
    where
        T: ExtensionMetaInfo,
    {
        let extension_name = T::name();
        let extension_type = T::extension_type();

        info!(
            "remove extension: {}, type: {}",
            extension_name,
            extension_type.as_str()
        );

        let (tx, rx) = oneshot::channel();

        let send = self
            .sender
            .send(ExtensionOpt::Remove(
                extension_name.clone(),
                extension_type,
                tx,
            ))
            .await;

        let Ok(_) = send else {
            let err_msg = format!("remove extension {} failed", extension_name);
            return Err(RemoveExtensionError::new(err_msg).into());
        };

        let ret = rx.await;

        match ret {
            Ok(_) => Ok(()),
            Err(_) => {
                let err_msg = format!("remove extension {} failed", extension_name);
                Err(RemoveExtensionError::new(err_msg).into())
            }
        }
    }

    pub async fn load_registry(&self, url: Url) -> Result<RegistryProxy, StdError> {
        let url_str = url.to_string();
        info!("load registry extension: {}", url_str);

        let (tx, rx) = oneshot::channel();

        let send = self
            .sender
            .send(ExtensionOpt::Load(url, ExtensionType::Registry, tx))
            .await;

        let Ok(_) = send else {
            let err_msg = format!("load registry extension failed: {}", url_str);
            return Err(LoadExtensionError::new(err_msg).into());
        };

        let extensions = rx.await;

        let Ok(extension) = extensions else {
            let err_msg = format!("load registry extension failed: {}", url_str);
            return Err(LoadExtensionError::new(err_msg).into());
        };

        let Ok(extensions) = extension else {
            let err_msg = format!("load registry extension failed: {}", url_str);
            return Err(LoadExtensionError::new(err_msg).into());
        };

        match extensions {
            Extensions::Registry(proxy) => Ok(proxy),
            _ => {
                panic!("load registry extension failed: invalid extension type");
            }
        }
    }

    pub async fn load_invoker(&self, url: Url) -> Result<InvokerProxy, StdError> {
        let url_str = url.to_string();
        info!("load invoker extension: {}", url_str);

        let (tx, rx) = oneshot::channel();

        let send = self
            .sender
            .send(ExtensionOpt::Load(url, ExtensionType::Invoker, tx))
            .await;

        let Ok(_) = send else {
            let err_msg = format!("load invoker extension failed: {}", url_str);
            return Err(LoadExtensionError::new(err_msg).into());
        };

        let extensions = rx.await;

        let Ok(extension) = extensions else {
            let err_msg = format!("load invoker extension failed: {}", url_str);
            return Err(LoadExtensionError::new(err_msg).into());
        };

        let Ok(extensions) = extension else {
            let err_msg = format!("load invoker extension failed: {}", url_str);
            return Err(LoadExtensionError::new(err_msg).into());
        };

        match extensions {
            Extensions::Invoker(proxy) => Ok(proxy),
            _ => {
                panic!("load invoker extension failed: invalid extension type");
            }
        }
    }
}

enum ExtensionOpt {
    Register(
        String,
        ExtensionFactories,
        ExtensionType,
        oneshot::Sender<Result<(), StdError>>,
    ),
    Remove(String, ExtensionType, oneshot::Sender<Result<(), StdError>>),
    Load(
        Url,
        ExtensionType,
        oneshot::Sender<Result<Extensions, StdError>>,
    ),
}

#[allow(private_bounds)]
#[async_trait::async_trait]
pub trait Extension {
    type Target;

    fn name() -> String;

    async fn create(url: Url) -> Result<Self::Target, StdError>;
}

#[allow(private_bounds)]
pub(crate) trait ExtensionMetaInfo {
    fn name() -> String;
    fn extension_type() -> ExtensionType;
    fn extension_factory() -> ExtensionFactories;
}

pub(crate) enum Extensions {
    Registry(RegistryProxy),
    Invoker(InvokerProxy),
}

pub(crate) enum ExtensionFactories {
    RegistryExtensionFactory(registry_extension::RegistryExtensionFactory),
    InvokerExtensionFactory(invoker_extension::InvokerExtensionFactory),
}

#[derive(Error, Debug)]
#[error("{0}")]
pub(crate) struct RegisterExtensionError(String);

impl RegisterExtensionError {
    pub fn new(msg: String) -> Self {
        RegisterExtensionError(msg)
    }
}

#[derive(Error, Debug)]
#[error("{0}")]
pub struct RemoveExtensionError(String);

impl RemoveExtensionError {
    pub fn new(msg: String) -> Self {
        RemoveExtensionError(msg)
    }
}

#[derive(Error, Debug)]
#[error("{0}")]
pub struct LoadExtensionError(String);

impl LoadExtensionError {
    pub fn new(msg: String) -> Self {
        LoadExtensionError(msg)
    }
}

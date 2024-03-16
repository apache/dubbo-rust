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

pub mod registry_extension;

use crate::{
    extension::registry_extension::proxy::RegistryProxy, registry::registry::StaticRegistry,
};
use dubbo_base::{extension_param::ExtensionType, url::UrlParam, StdError, Url};
use dubbo_logger::tracing::{error, info};
use std::{future::Future, pin::Pin, sync::Arc};
use thiserror::Error;
use tokio::sync::{oneshot, Semaphore};

pub static EXTENSIONS: once_cell::sync::Lazy<ExtensionDirectoryCommander> =
    once_cell::sync::Lazy::new(|| ExtensionDirectory::init());

#[derive(Default)]
struct ExtensionDirectory {
    registry_extension_loader: registry_extension::RegistryExtensionLoader,
}

impl ExtensionDirectory {
    fn init() -> ExtensionDirectoryCommander {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<ExtensionOpt>(64);

        tokio::spawn(async move {
            let mut extension_directory = ExtensionDirectory::default();

            // register static registry extension
            let _ = extension_directory.register(
                StaticRegistry::name(),
                StaticRegistry::convert_to_extension_factories(),
                ExtensionType::Registry,
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
                                    error!("load extension failed: {}", err);
                                    let _ = callback.send(Err(err));
                                }
                            }
                        });
                    }
                    Err(err) => {
                        error!("load extension failed: {}", err);
                        let _ = callback.send(Err(err));
                    }
                }
            }
        }
    }
}

pub(crate) struct LoadExtensionPromise<T> {
    extension: Arc<Option<T>>,
    fut: Option<Pin<Box<dyn Future<Output = Result<T, StdError>> + Send + 'static>>>,
    semaphore: Arc<Semaphore>,
}

impl<T> LoadExtensionPromise<T>
where
    T: Send + Clone + 'static,
{
    pub(crate) fn new(
        fut: Pin<Box<dyn Future<Output = Result<T, StdError>> + Send + 'static>>,
    ) -> Self {
        LoadExtensionPromise {
            extension: Arc::new(None),
            fut: Some(fut),
            semaphore: Arc::new(Semaphore::new(0)),
        }
    }

    fn get_extension(&self) -> Option<T> {
        self.extension.as_ref().as_ref().map(|a| a.clone())
    }

    pub(crate) async fn resolve(&mut self) -> Result<T, StdError> {
        let extension = self.get_extension();
        if let Some(extension) = extension {
            return Ok(extension);
        }

        let fut = self.fut.take();
        let Some(mut fut) = fut else {
            let _ = self.semaphore.acquire().await;
            // check it again
            let extension = self.get_extension();
            if let Some(extension) = extension {
                info!("promise has been resolved.");
                return Ok(extension);
            }
            return Err(LoadExtensionError::new("load extension canceled ".to_string()).into());
        };

        match fut.as_mut().await {
            Ok(extension) => {
                info!("create extension success");
                let ptr = Arc::as_ptr(&self.extension) as *mut Option<T>;
                unsafe {
                    *ptr = Some(extension.clone());
                }
                self.semaphore.close();
                Ok(extension)
            }
            Err(err) => {
                error!("create extension failed: {}", err);
                self.semaphore.close();
                Err(LoadExtensionError::new(
                    "load extension failed, create extension occur an error".to_string(),
                )
                .into())
            }
        }
    }
}

impl<T> Clone for LoadExtensionPromise<T> {
    fn clone(&self) -> Self {
        LoadExtensionPromise {
            extension: self.extension.clone(),
            fut: None,
            semaphore: self.semaphore.clone(),
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
        T: Extension,
        T: ExtensionMetaInfo,
        T: ConvertToExtensionFactories,
    {
        let extension_name = T::name();
        let extension_factories = T::convert_to_extension_factories();
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
        T: Extension,
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

pub(crate) trait Sealed {}

#[allow(private_bounds)]
#[async_trait::async_trait]
pub trait Extension: Sealed {
    type Target;

    fn name() -> String;

    async fn create(url: Url) -> Result<Self::Target, StdError>;
}

#[allow(private_bounds)]
pub(crate) trait ExtensionMetaInfo {
    fn extension_type() -> ExtensionType;
}

pub(crate) enum Extensions {
    Registry(RegistryProxy),
}

pub(crate) enum ExtensionFactories {
    RegistryExtensionFactory(registry_extension::RegistryExtensionFactory),
}

#[allow(private_bounds)]
pub(crate) trait ConvertToExtensionFactories {
    fn convert_to_extension_factories() -> ExtensionFactories;
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

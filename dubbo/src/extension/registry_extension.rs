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

use std::{collections::HashMap, future::Future, pin::Pin};

use async_trait::async_trait;
use thiserror::Error;
use tokio::sync::mpsc::Receiver;
use tower::discover::Change;

use dubbo_base::{
    extension_param::ExtensionName, registry_param::RegistryUrl, url::UrlParam, StdError, Url,
};
use proxy::RegistryProxy;

use crate::extension::{
    ConvertToExtensionFactories, Extension, ExtensionFactories, ExtensionMetaInfo, ExtensionType,
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

pub(crate) struct RegistryExtensionFactory {
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

    use dubbo_base::{StdError, Url};
    use dubbo_logger::tracing::error;

    use crate::extension::registry_extension::{DiscoverStream, Registry};

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

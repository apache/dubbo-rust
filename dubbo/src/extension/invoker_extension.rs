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

use crate::{
    extension::{
        invoker_extension::proxy::InvokerProxy, Extension, ExtensionFactories, ExtensionMetaInfo,
        LoadExtensionPromise,
    },
    params::extension_param::{ExtensionName, ExtensionType},
    url::UrlParam,
    StdError, Url,
};
use async_trait::async_trait;
use bytes::Bytes;
use futures_core::Stream;
use std::{collections::HashMap, future::Future, marker::PhantomData, pin::Pin};
use thiserror::Error;

#[async_trait]
pub trait Invoker {
    async fn invoke(
        &self,
        invocation: GrpcInvocation,
    ) -> Result<Pin<Box<dyn Stream<Item = Bytes> + Send + 'static>>, StdError>;

    async fn url(&self) -> Result<Url, StdError>;
}

pub enum CallType {
    Unary,
    ClientStream,
    ServerStream,
    BiStream,
}

pub struct GrpcInvocation {
    service_name: String,
    method_name: String,
    arguments: Vec<Argument>,
    attachments: HashMap<String, String>,
    call_type: CallType,
}

pub struct Argument {
    name: String,
    value: Box<dyn Stream<Item = Box<dyn Serializable + Send + 'static>> + Send + 'static>,
}

pub trait Serializable {
    fn serialize(&self, serialization_type: String) -> Result<Bytes, StdError>;
}

pub trait Deserializable {
    fn deserialize(&self, bytes: Bytes, deserialization_type: String) -> Result<Self, StdError>
    where
        Self: Sized;
}

pub mod proxy {
    use crate::{
        extension::invoker_extension::{GrpcInvocation, Invoker},
        StdError, Url,
    };
    use async_trait::async_trait;
    use bytes::Bytes;
    use futures_core::Stream;
    use std::pin::Pin;
    use thiserror::Error;
    use tokio::sync::{mpsc::Sender, oneshot};
    use tracing::error;

    pub(super) enum InvokerOpt {
        Invoke(
            GrpcInvocation,
            oneshot::Sender<Result<Pin<Box<dyn Stream<Item = Bytes> + Send + 'static>>, StdError>>,
        ),
        Url(oneshot::Sender<Result<Url, StdError>>),
    }

    #[derive(Clone)]
    pub struct InvokerProxy {
        tx: Sender<InvokerOpt>,
    }

    #[async_trait]
    impl Invoker for InvokerProxy {
        async fn invoke(
            &self,
            invocation: GrpcInvocation,
        ) -> Result<Pin<Box<dyn Stream<Item = Bytes> + Send + 'static>>, StdError> {
            let (tx, rx) = oneshot::channel();
            let ret = self.tx.send(InvokerOpt::Invoke(invocation, tx)).await;
            match ret {
                Ok(_) => {}
                Err(err) => {
                    error!(
                        "call invoke method failed by invoker proxy, error: {:?}",
                        err
                    );
                    return Err(InvokerProxyError::new(
                        "call invoke method failed by invoker proxy",
                    )
                    .into());
                }
            }
            let ret = rx.await?;
            ret
        }

        async fn url(&self) -> Result<Url, StdError> {
            let (tx, rx) = oneshot::channel();
            let ret = self.tx.send(InvokerOpt::Url(tx)).await;
            match ret {
                Ok(_) => {}
                Err(err) => {
                    error!("call url method failed by invoker proxy, error: {:?}", err);
                    return Err(
                        InvokerProxyError::new("call url method failed by invoker proxy").into(),
                    );
                }
            }
            let ret = rx.await?;
            ret
        }
    }

    impl From<Box<dyn Invoker + Send + 'static>> for InvokerProxy {
        fn from(invoker: Box<dyn Invoker + Send + 'static>) -> Self {
            let (tx, mut rx) = tokio::sync::mpsc::channel(64);
            tokio::spawn(async move {
                while let Some(opt) = rx.recv().await {
                    match opt {
                        InvokerOpt::Invoke(invocation, tx) => {
                            let result = invoker.invoke(invocation).await;
                            let callback_ret = tx.send(result);
                            match callback_ret {
                                Ok(_) => {}
                                Err(Err(err)) => {
                                    error!("invoke method has been called, but callback to caller failed. {:?}", err);
                                }
                                _ => {}
                            }
                        }
                        InvokerOpt::Url(tx) => {
                            let ret = tx.send(invoker.url().await);
                            match ret {
                                Ok(_) => {}
                                Err(err) => {
                                    error!("url method has been called, but callback to caller failed. {:?}", err);
                                }
                            }
                        }
                    }
                }
            });
            InvokerProxy { tx }
        }
    }

    #[derive(Error, Debug)]
    #[error("invoker proxy error: {0}")]
    pub struct InvokerProxyError(String);

    impl InvokerProxyError {
        pub fn new(msg: &str) -> Self {
            InvokerProxyError(msg.to_string())
        }
    }
}

#[derive(Default)]
pub(super) struct InvokerExtensionLoader {
    factories: HashMap<String, InvokerExtensionFactory>,
}

impl InvokerExtensionLoader {
    pub fn register(&mut self, extension_name: String, factory: InvokerExtensionFactory) {
        self.factories.insert(extension_name, factory);
    }

    pub fn remove(&mut self, extension_name: String) {
        self.factories.remove(&extension_name);
    }

    pub fn load(&mut self, url: Url) -> Result<LoadExtensionPromise<InvokerProxy>, StdError> {
        let extension_name = url.query::<ExtensionName>();
        let Some(extension_name) = extension_name else {
            return Err(InvokerExtensionLoaderError::new(
                "load invoker extension failed, extension mustn't be empty",
            )
            .into());
        };
        let extension_name = extension_name.value();
        let factory = self.factories.get_mut(&extension_name);
        let Some(factory) = factory else {
            let err_msg = format!(
                "load {} invoker extension failed, can not found extension factory",
                extension_name
            );
            return Err(InvokerExtensionLoaderError(err_msg).into());
        };
        factory.create(url)
    }
}

type InvokerExtensionConstructor = fn(
    Url,
) -> Pin<
    Box<dyn Future<Output = Result<Box<dyn Invoker + Send + 'static>, StdError>> + Send + 'static>,
>;
pub(crate) struct InvokerExtensionFactory {
    constructor: InvokerExtensionConstructor,
    instances: HashMap<String, LoadExtensionPromise<InvokerProxy>>,
}

impl InvokerExtensionFactory {
    pub fn new(constructor: InvokerExtensionConstructor) -> Self {
        Self {
            constructor,
            instances: HashMap::default(),
        }
    }
}

impl InvokerExtensionFactory {
    pub fn create(&mut self, url: Url) -> Result<LoadExtensionPromise<InvokerProxy>, StdError> {
        let key = url.to_string();

        match self.instances.get(&key) {
            Some(instance) => Ok(instance.clone()),
            None => {
                let constructor = self.constructor;
                let creator = move |url: Url| {
                    let invoker_future = constructor(url);
                    Box::pin(async move {
                        let invoker = invoker_future.await?;
                        Ok(InvokerProxy::from(invoker))
                    })
                        as Pin<
                            Box<
                                dyn Future<Output = Result<InvokerProxy, StdError>>
                                    + Send
                                    + 'static,
                            >,
                        >
                };

                let promise: LoadExtensionPromise<InvokerProxy> =
                    LoadExtensionPromise::new(Box::new(creator), url);
                self.instances.insert(key, promise.clone());
                Ok(promise)
            }
        }
    }
}

pub struct InvokerExtension<T>(PhantomData<T>)
where
    T: Invoker + Send + 'static;

impl<T> ExtensionMetaInfo for InvokerExtension<T>
where
    T: Invoker + Send + 'static,
    T: Extension<Target = Box<dyn Invoker + Send + 'static>>,
{
    fn name() -> String {
        T::name()
    }

    fn extension_type() -> ExtensionType {
        ExtensionType::Invoker
    }

    fn extension_factory() -> ExtensionFactories {
        ExtensionFactories::InvokerExtensionFactory(InvokerExtensionFactory::new(
            <T as Extension>::create,
        ))
    }
}

#[derive(Error, Debug)]
#[error("{0}")]
pub struct InvokerExtensionLoaderError(String);

impl InvokerExtensionLoaderError {
    pub fn new(msg: &str) -> Self {
        InvokerExtensionLoaderError(msg.to_string())
    }
}

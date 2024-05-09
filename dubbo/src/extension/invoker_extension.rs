use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use async_trait::async_trait;
use bytes::Bytes;
use futures_core::Stream;
use crate::{StdError, Url};
use crate::extension::{ConvertToExtensionFactories, Extension, ExtensionFactories, ExtensionMetaInfo, LoadExtensionPromise};
use crate::extension::invoker_extension::proxy::InvokerProxy;
use crate::params::extension_param::ExtensionType;


#[async_trait]
pub trait Invoker {

    async fn invoke(&self, invocation: GrpcInvocation) -> Result<Pin<Box<dyn Stream<Item= Bytes> + Send + 'static>>, StdError>;

    async fn url(&self) -> Url;

}


pub struct GrpcInvocation {
    service_name: String,
    method_name: String,
    arguments: Vec<Argument>,
    attachments: HashMap<String, String>
}

pub struct Argument {
    name: String,
    value: Box<dyn Serializable>
}


pub trait Serializable {
    fn serialize(&self, serialization_type: String) -> Bytes;
}


pub trait Deserializable {
    fn deserialize(&self, bytes: Bytes, deserialization_type: String) -> Self;
}

pub mod proxy {
    use std::pin::Pin;
    use async_trait::async_trait;
    use bytes::Bytes;
    use futures_core::Stream;
    use tokio::sync::mpsc::Sender;
    use tokio::sync::oneshot;
    use crate::extension::invoker_extension::{GrpcInvocation, Invoker};
    use crate::{StdError, Url};

    pub(super) enum InvokerOpt {
        Invoke(GrpcInvocation, oneshot::Sender<Result<Pin<Box<dyn Stream<Item= Bytes> + Send + 'static>>, StdError>>),
        Url(oneshot::Sender<Url>)
    }

    #[derive(Clone)]
    pub struct InvokerProxy {
        tx: Sender<InvokerOpt>
    }

    impl InvokerProxy {

        pub fn new(invoker: Box<dyn Invoker + Send + 'static>) -> Self {
            let (tx, mut rx) = tokio::sync::mpsc::channel(64);
            tokio::spawn(async move {
                while let Some(opt) = rx.recv().await {
                    match opt {
                        InvokerOpt::Invoke(invocation, tx) => {
                            let result = invoker.invoke(invocation);
                            let _ = tx.send(result);
                        },
                        InvokerOpt::Url(tx) => {
                            let _ = tx.send(invoker.url());
                        }
                    }
                }
            });

            InvokerProxy {
                tx
            }
        }

    }

    #[async_trait]
    impl Invoker for InvokerProxy {
        async fn invoke(&self, invocation: GrpcInvocation) -> Result<Pin<Box<dyn Stream<Item=Bytes> + Send + 'static>>, StdError> {
            let (tx, rx) = oneshot::channel();
            let _ = self.tx.send(InvokerOpt::Invoke(invocation, tx));
            let ret = rx.await?;
            ret
        }

        async fn url(&self) -> Url {
            let (tx, rx) = oneshot::channel();
            let _ = self.tx.send(InvokerOpt::Url(tx));
            let ret = rx.await?;
            ret
        }
    }
}


type InvokerExtensionConstructor = Box<dyn Fn(Url) -> Pin<Box<dyn Future<Output=Result<Box<dyn Invoker + Send + 'static>, StdError>>>>>;
pub(crate) struct InvokerExtensionFactory {
    constructor: InvokerExtensionConstructor,
    instances: HashMap<String, LoadExtensionPromise<InvokerProxy>>
}




impl<T> crate::extension::Sealed for T where T: Invoker + Send + 'static {}

impl<T> ExtensionMetaInfo for T
where
    T: Invoker + Send + 'static,
    T: Extension<Target= Box<dyn Invoker + Send + 'static>>
{
    fn extension_type() -> ExtensionType {
        ExtensionType::Invoker
    }
}


impl<T> ConvertToExtensionFactories for T
where
    T: Invoker + Send + 'static,
    T: Extension<Target= Box<dyn Invoker + Send + 'static>>
{
    fn convert_to_extension_factories() -> ExtensionFactories {
        todo!()
    }
}

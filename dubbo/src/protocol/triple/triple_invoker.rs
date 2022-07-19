use crate::common::url::Url;
use crate::protocol::Invoker;

#[allow(dead_code)]
#[derive(Clone, Default)]
pub struct TripleInvoker {
    url: Url,
}

impl TripleInvoker {
    pub fn new(url: Url) -> TripleInvoker {
        Self { url }
    }
}

impl Invoker for TripleInvoker {
    fn invoke<M1>(
        &self,
        _req: crate::protocol::invocation::Request<M1>,
    ) -> crate::protocol::invocation::Response<String>
    where
        M1: Send + 'static,
    {
        todo!()
    }

    fn is_available(&self) -> bool {
        todo!()
    }

    fn destroy(&self) {
        todo!()
    }

    fn get_url(&self) -> Url {
        todo!()
    }
}

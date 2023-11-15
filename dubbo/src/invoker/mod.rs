use dubbo_base::Url;
use tower_service::Service;

use crate::{codegen::TripleInvoker, param::Param, svc::NewService};

#[derive(Clone)]
pub struct NewInvoker {
    url: Url
}

pub enum InvokerComponent {
    TripleInvoker(TripleInvoker)
}


impl NewInvoker {
    pub fn new(url: Url) -> Self {
        Self {
            url
        }
    }
}

impl From<String> for NewInvoker {
    fn from(url: String) -> Self {
        Self {
            url: Url::from_url(&url).unwrap()
        }
    }
}

impl Param<Url> for NewInvoker {
    fn param(&self) -> Url {
        self.url.clone()
    }
}

impl NewService<()> for NewInvoker {
    type Service = InvokerComponent;
    fn new_service(&self, _: ()) -> Self::Service {
        // todo create another invoker
        InvokerComponent::TripleInvoker(TripleInvoker::new(self.url.clone()))
    }
}


impl<B> Service<http::Request<B>> for InvokerComponent 
where
    B: http_body::Body + Unpin + Send + 'static,
    B::Error: Into<crate::Error>,
    B::Data: Send + Unpin,
{
    type Response = http::Response<crate::BoxBody>;

    type Error = crate::Error;

    type Future = crate::BoxFuture<Self::Response, Self::Error>;

    fn call(&mut self, req: http::Request<B>) -> Self::Future {
       match self {
           InvokerComponent::TripleInvoker(invoker) => invoker.call(req),
       }
    }

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        match self {
            InvokerComponent::TripleInvoker(invoker) => <TripleInvoker as Service<http::Request<B>>>::poll_ready(invoker, cx),
        }
    }
}

// InvokerComponent::TripleInvoker(invoker) => <TripleInvoker as Service<http::Request<B>>>::poll_ready(invoker, cx),
use dubbo_base::Url;

use crate::{codegen::TripleInvoker, svc::NewService, invoker::clone_invoker::CloneInvoker};

pub mod clone_body;
pub mod clone_invoker;


pub struct NewInvoker;


impl NewService<String> for NewInvoker {
    type Service = CloneInvoker<TripleInvoker>;

    fn new_service(&self, url: String) -> Self::Service {
        // todo create another invoker by url protocol

        let url = Url::from_url(&url).unwrap();
        CloneInvoker::new(TripleInvoker::new(url))
    }
}
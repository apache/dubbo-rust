use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug)]
pub struct Invocation {
    pub method_name: String,
    pub service_name: String,
    pub attachments: HashMap<String, String>,
}

impl Invocation {
    pub fn new(method_name: String, service_name: String, attachments: HashMap<String, String>) -> Invocation {
        return Invocation {
            method_name,
            service_name,
            attachments,
        };
    }
}

pub trait RpcInvocation {
    fn method_name(&self) -> String;

    fn service_name(&self) -> String;

    fn attachments(&self) -> HashMap<String, String>;
}

impl RpcInvocation for Invocation {
    fn method_name(&self) -> String {
        self.method_name.clone()
    }

    fn service_name(&self) -> String {
        self.service_name.clone()
    }

    fn attachments(&self) -> HashMap<String, String> {
        self.attachments.clone()
    }
}


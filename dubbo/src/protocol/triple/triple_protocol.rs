use std::collections::HashMap;

use async_trait::async_trait;

use super::triple_exporter::TripleExporter;
use super::triple_invoker::TripleInvoker;
use super::triple_server::TripleServer;
use crate::common::url::Url;
use crate::protocol::Protocol;

pub struct TripleProtocol {
    servers: HashMap<String, TripleServer>,
}

impl Default for TripleProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl TripleProtocol {
    pub fn new() -> Self {
        TripleProtocol {
            servers: HashMap::new(),
        }
    }

    pub fn get_server(&self, name: String) -> Option<TripleServer> {
        self.servers.get(&name).map(|data| data.to_owned())
    }
}

#[async_trait]
impl Protocol for TripleProtocol {
    type Invoker = TripleInvoker;

    type Exporter = TripleExporter;

    fn destroy(&self) {
        todo!()
    }

    async fn export(self, url: Url) -> Self::Exporter {
        let server = TripleServer::new(url.service_key.clone());
        server.serve(url.url.clone()).await;

        TripleExporter::new()
    }

    async fn refer(self, url: Url) -> Self::Invoker {
        TripleInvoker::new(url)
        // Self::Invoker
    }
}

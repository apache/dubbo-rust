
use std::collections::HashMap;

use super::grpc_invoker::GrpcInvoker;
use super::grpc_exporter::GrpcExporter;
use crate::common::url::Url;
use crate::service::protocol::Protocol;
use super::grpc_server::GrpcServer;

pub struct GrpcProtocol {
    server_map: HashMap<String, GrpcServer>,
    export_map: HashMap<String, GrpcExporter<GrpcInvoker>>
}

impl GrpcProtocol {
    pub fn new() -> Self {
        Self {
            server_map: HashMap::new(),
            export_map: HashMap::new(),
        }
    }
}

impl Default for GrpcProtocol {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Protocol for GrpcProtocol
{
    type Invoker = GrpcInvoker;

    type Exporter = GrpcExporter<Self::Invoker>;

    fn destroy(&self) {
        todo!()
    }

    async fn refer(&self, url: Url) -> Self::Invoker {
        GrpcInvoker::new(url)
    }

    async fn export(self, url: Url) -> Self::Exporter {
        let service_key = url.service_key.clone();

        let exporter: GrpcExporter<GrpcInvoker> = GrpcExporter::new(service_key.clone(), GrpcInvoker::new(url.clone()));
        let mut export = self.export_map;
        export.insert(service_key.clone(), exporter.clone());

        // 启动服务

        let server = super::grpc_server::GrpcServer::new(service_key.clone());
        let mut server_map = self.server_map;
        server_map.insert(service_key.clone(), server.clone());
        server.serve(url.clone()).await;
        exporter
    }
}
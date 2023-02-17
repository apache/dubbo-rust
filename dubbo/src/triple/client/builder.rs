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
    cluster::directory::StaticDirectory,
    codegen::{ClusterInvoker, Directory, RegistryDirectory},
    triple::compression::CompressionEncoding,
    utils::boxed::BoxService,
};

use super::TripleClient;

pub type ClientBoxService =
    BoxService<http::Request<hyper::Body>, http::Response<crate::BoxBody>, crate::Error>;

#[derive(Clone, Debug, Default)]
pub struct ClientBuilder {
    pub timeout: Option<u64>,
    pub connector: &'static str,
    directory: Option<Box<dyn Directory>>,
    cluster_invoker: Option<ClusterInvoker>,
}

impl ClientBuilder {
    pub fn new() -> ClientBuilder {
        ClientBuilder {
            timeout: None,
            connector: "",
            directory: None,
            cluster_invoker: None,
        }
    }

    pub fn from_static(host: &str) -> ClientBuilder {
        Self {
            timeout: None,
            connector: "",
            directory: Some(Box::new(StaticDirectory::new(&host))),
            cluster_invoker: None,
        }
    }

    pub fn from_uri(uri: &http::Uri) -> ClientBuilder {
        Self {
            timeout: None,
            connector: "",
            directory: Some(Box::new(StaticDirectory::from_uri(&uri))),
            cluster_invoker: None,
        }
    }

    pub fn with_timeout(self, timeout: u64) -> Self {
        Self {
            timeout: Some(timeout),
            ..self
        }
    }

    /// host: http://0.0.0.0:8888
    pub fn with_directory(self, directory: Box<dyn Directory>) -> Self {
        Self {
            directory: Some(directory),
            cluster_invoker: None,
            ..self
        }
    }

    pub fn with_registry_directory(self, registry: RegistryDirectory) -> Self {
        Self {
            directory: None,
            cluster_invoker: Some(ClusterInvoker::with_directory(registry)),
            ..self
        }
    }

    pub fn with_host(self, host: &'static str) -> Self {
        Self {
            directory: Some(Box::new(StaticDirectory::new(&host))),
            ..self
        }
    }

    pub fn with_connector(self, connector: &'static str) -> Self {
        Self {
            connector: connector,
            cluster_invoker: None,
            ..self
        }
    }

    pub fn build(self) -> TripleClient {
        TripleClient {
            send_compression_encoding: Some(CompressionEncoding::Gzip),
            directory: self.directory,
            cluster_invoker: self.cluster_invoker,
        }
    }
}

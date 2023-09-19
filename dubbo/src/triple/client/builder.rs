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

use std::sync::Arc;

use crate::{
    cluster::{directory::StaticDirectory, Cluster, Directory, MockCluster, MockDirectory},
    codegen::{RegistryDirectory, RpcInvocation, TripleInvoker},
    protocol::BoxInvoker,
    triple::compression::CompressionEncoding,
    utils::boxed_clone::BoxCloneService,
};

use aws_smithy_http::body::SdkBody;
use dubbo_base::Url;

use super::TripleClient;

pub type ClientBoxService =
    BoxCloneService<http::Request<SdkBody>, http::Response<crate::BoxBody>, crate::Error>;

#[derive(Clone, Debug, Default)]
pub struct ClientBuilder {
    pub timeout: Option<u64>,
    pub connector: &'static str,
    directory: Option<Box<dyn Directory>>,
    pub direct: bool,
    host: String,
}

impl ClientBuilder {
    pub fn new() -> ClientBuilder {
        ClientBuilder {
            timeout: None,
            connector: "",
            directory: None,
            direct: false,
            host: "".to_string(),
        }
    }

    pub fn from_static(host: &str) -> ClientBuilder {
        Self {
            timeout: None,
            connector: "",
            directory: Some(Box::new(StaticDirectory::new(&host))),
            direct: true,
            host: host.to_string(),
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
            ..self
        }
    }

    pub fn with_registry_directory(self, registry: RegistryDirectory) -> Self {
        Self {
            directory: Some(Box::new(registry)),
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
            ..self
        }
    }

    pub fn with_direct(self, direct: bool) -> Self {
        Self { direct, ..self }
    }

    pub(crate) fn direct_build(self) -> TripleClient {
        let mut cli = TripleClient {
            send_compression_encoding: Some(CompressionEncoding::Gzip),
            builder: Some(self.clone()),
            invoker: None,
        };
        cli.invoker = Some(Box::new(TripleInvoker::new(
            Url::from_url(&self.host).unwrap(),
        )));
        return cli;
    }

    pub fn build(self, invocation: Arc<RpcInvocation>) -> Option<BoxInvoker> {
        if self.direct {
            return Some(Box::new(TripleInvoker::new(
                Url::from_url(&self.host).unwrap(),
            )));
        }
        let invokers = match self.directory {
            Some(v) => v.list(invocation),
            None => panic!("use direct connection"),
        };

        let cluster = MockCluster::default().join(Box::new(MockDirectory::new(invokers)));

        return Some(cluster);
    }
}

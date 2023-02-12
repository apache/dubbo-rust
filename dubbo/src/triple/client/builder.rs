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

use http::Uri;
use hyper::client::conn::Builder;
use tokio::time::Duration;
use tower::ServiceBuilder;

use crate::triple::transport::connection::Connection;
use crate::utils::boxed::BoxService;

pub type ClientBoxService =
    BoxService<http::Request<hyper::Body>, http::Response<crate::BoxBody>, crate::Error>;

#[derive(Clone, Debug, Default)]
pub struct ClientBuilder {
    pub uri: Uri,
    pub timeout: Option<u64>,
    pub connector: &'static str,
}

impl ClientBuilder {
    pub fn new() -> ClientBuilder {
        ClientBuilder {
            uri: Uri::builder().build().unwrap(),
            timeout: None,
            connector: "",
        }
    }

    pub fn from_static(s: &'static str) -> ClientBuilder {
        Self::from(Uri::from_static(s))
    }

    pub fn with_timeout(self, timeout: u64) -> Self {
        Self {
            timeout: Some(timeout),
            ..self
        }
    }

    pub fn with_host(self, host: &'static str) -> Self {
        Self {
            uri: Uri::from_static(host),
            ..self
        }
    }

    pub fn with_connector(self, connector: &'static str) -> Self {
        Self {
            connector: connector,
            ..self
        }
    }

    pub fn connect(self) -> ClientBoxService {
        let builder = ServiceBuilder::new();
        let timeout = self.timeout.unwrap_or(5);
        let builder = builder.timeout(Duration::from_secs(timeout));

        let mut b = Builder::new();
        let hyper_builder = b.http2_only(true);
        let conn = Connection::new()
            .with_host(self.uri.clone())
            .with_connector(self.connector)
            .with_builder(hyper_builder.to_owned());

        BoxService::new(builder.service(conn))
    }
}

impl From<Uri> for ClientBuilder {
    fn from(u: Uri) -> Self {
        Self {
            uri: u,
            timeout: None,
            connector: "tcp",
        }
    }
}

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

pub mod http_connector;
pub mod https_connector;
#[cfg(any(target_os = "macos", target_os = "unix"))]
pub mod unix_connector;

use hyper::Uri;
use tokio::io::{AsyncRead, AsyncWrite};
use tower::make::MakeConnection;
use tower_service::Service;

use super::io::BoxIO;
use crate::utils::boxed_clone::BoxCloneService;

#[derive(Clone)]
pub struct Connector<C> {
    inner: C,
}

impl<C> Connector<C> {
    pub fn new(inner: C) -> Connector<C>
    where
        C: Service<Uri>,
        C::Error: Into<crate::Error>,
        C::Response: AsyncRead + AsyncWrite + Send + 'static,
    {
        Self { inner }
    }
}

impl<C> Service<Uri> for Connector<C>
where
    C: MakeConnection<Uri>,
    C::Connection: Unpin + Send + 'static,
    C::Future: Send + 'static,
    crate::Error: From<C::Error> + Send + 'static,
{
    type Response = BoxIO;

    type Error = crate::Error;

    type Future = crate::BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        MakeConnection::poll_ready(&mut self.inner, cx).map_err(Into::into)
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        let conn = self.inner.make_connection(uri);

        Box::pin(async move {
            let io = conn.await?;
            Ok(BoxIO::new(io))
        })
    }
}

pub fn get_connector(connector: &str) -> BoxCloneService<Uri, BoxIO, crate::Error> {
    match connector {
        "http" => {
            let c = http_connector::HttpConnector::new();
            BoxCloneService::new(Connector::new(c))
        }
        "https" => {
            let c = https_connector::HttpsConnector::new();
            BoxCloneService::new(Connector::new(c))
        }
        #[cfg(any(target_os = "macos", target_os = "unix"))]
        "unix" => {
            let c = unix_connector::UnixConnector::new();
            BoxCloneService::new(Connector::new(c))
        }
        _ => {
            let c = http_connector::HttpConnector::new();
            BoxCloneService::new(Connector::new(c))
        }
    }
}

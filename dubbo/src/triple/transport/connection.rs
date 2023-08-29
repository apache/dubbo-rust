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

use std::task::Poll;

use dubbo_logger::tracing::debug;
use hyper::client::{conn::Builder, service::Connect};
use tower_service::Service;

use crate::{boxed, triple::transport::connector::get_connector};

#[derive(Debug, Clone)]
pub struct Connection {
    host: hyper::Uri,
    connector: String,
    builder: Builder,
}

impl Default for Connection {
    fn default() -> Self {
        Self::new()
    }
}

impl Connection {
    pub fn new() -> Self {
        Connection {
            host: hyper::Uri::default(),
            connector: "http".to_string(),
            builder: Builder::new(),
        }
    }

    pub fn with_connector(mut self, connector: String) -> Self {
        self.connector = connector;
        self
    }

    pub fn with_host(mut self, uri: hyper::Uri) -> Self {
        self.host = uri;
        self
    }

    pub fn with_builder(mut self, builder: Builder) -> Self {
        self.builder = builder;
        self
    }
}

impl<ReqBody> Service<http::Request<ReqBody>> for Connection
where
    ReqBody: http_body::Body + Unpin + Send + 'static,
    ReqBody::Data: Send + Unpin,
    ReqBody::Error: Into<crate::Error>,
{
    type Response = http::Response<crate::BoxBody>;

    type Error = crate::Error;

    type Future = crate::BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        let builder = self.builder.clone().http2_only(true).to_owned();
        let mut connector = Connect::new(get_connector(self.connector.as_str()), builder);
        let uri = self.host.clone();
        let fut = async move {
            debug!("send base call to {}", uri);
            let mut con = connector.call(uri).await.unwrap();

            con.call(req)
                .await
                .map_err(|err| err.into())
                .map(|res| res.map(boxed))
        };

        Box::pin(fut)
    }
}

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

use hyper::client::{conn::Builder, service::Connect};
use tower_service::Service;

use crate::{
    boxed, invoker::clone_body::CloneBody, triple::transport::connector::get_connector, StdError,
};

type HyperConnect = Connect<
    crate::utils::boxed_clone::BoxCloneService<http::Uri, super::io::BoxIO, StdError>,
    CloneBody,
    http::Uri,
>;

pub struct Connection {
    host: hyper::Uri,
    connector: &'static str,
    builder: Builder,
    connect: Option<HyperConnect>,
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
            connector: "http",
            builder: Builder::new(),
            connect: None,
        }
    }

    pub fn with_connector(mut self, connector: &'static str) -> Self {
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

    pub fn build(mut self) -> Self {
        let builder = self.builder.clone().http2_only(true).to_owned();
        let hyper_connect: HyperConnect = Connect::new(get_connector(self.connector), builder);
        self.connect = Some(hyper_connect);
        self
    }
}

impl Service<http::Request<CloneBody>> for Connection {
    type Response = http::Response<crate::BoxBody>;

    type Error = crate::Error;

    type Future = crate::BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        match self.connect {
            None => {
                panic!("connection must be built before use")
            }
            Some(ref mut connect) => connect.poll_ready(cx).map_err(|e| e.into()),
        }
    }

    fn call(&mut self, req: http::Request<CloneBody>) -> Self::Future {
        match self.connect {
            None => {
                panic!("connection must be built before use")
            }
            Some(ref mut connect) => {
                let uri = self.host.clone();
                let call_fut = connect.call(uri);
                let fut = async move {
                    let mut con = call_fut.await.unwrap();
                    con.call(req)
                        .await
                        .map_err(|err| err.into())
                        .map(|res| res.map(boxed))
                };

                return Box::pin(fut);
            }
        }
    }
}

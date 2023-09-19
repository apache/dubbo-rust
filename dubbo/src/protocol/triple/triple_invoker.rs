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

use aws_smithy_http::body::SdkBody;
use dubbo_base::Url;
use std::{
    fmt::{Debug, Formatter},
    str::FromStr,
};
use tower_service::Service;

use crate::{
    protocol::Invoker,
    triple::{client::builder::ClientBoxService, transport::connection::Connection},
    utils::boxed_clone::BoxCloneService,
};

#[derive(Clone)]
pub struct TripleInvoker {
    url: Url,
    conn: ClientBoxService,
}

impl TripleInvoker {
    pub fn new(url: Url) -> TripleInvoker {
        let uri = http::Uri::from_str(&url.to_url()).unwrap();
        let mut conn = Connection::new().with_host(uri.clone());
        if let Some(scheme) = uri.scheme_str() {
            conn = conn.with_connector(scheme.to_string());
        }
        Self {
            url,
            conn: BoxCloneService::new(conn),
        }
    }
}

impl Debug for TripleInvoker {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self.url).as_str())
    }
}

impl Invoker<http::Request<SdkBody>> for TripleInvoker {
    type Response = http::Response<crate::BoxBody>;

    type Error = crate::Error;

    type Future = crate::BoxFuture<Self::Response, Self::Error>;

    fn get_url(&self) -> Url {
        self.url.clone()
    }

    fn call(&mut self, req: http::Request<SdkBody>) -> Self::Future {
        self.conn.call(req)
    }

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.conn.poll_ready(cx)
    }
}

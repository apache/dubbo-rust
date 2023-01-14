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

use std::str::FromStr;

use tower_service::Service;

use crate::common::url::Url;
use crate::protocol::Invoker;
use crate::triple::client::connection::Connection;

#[allow(dead_code)]
#[derive(Clone, Default,Debug)]
pub struct TripleInvoker {
    url: Url,
    conn: Connection,
}

impl TripleInvoker {
    pub fn new(url: Url) -> TripleInvoker {
        let uri = http::Uri::from_str(&url.to_url()).unwrap();
        Self {
            url,
            conn: Connection::new().with_host(uri),
        }
    }
}

impl<ReqBody> Invoker<http::Request<ReqBody>> for TripleInvoker
where
    ReqBody: http_body::Body + Unpin + Send + 'static,
    ReqBody::Error: Into<crate::Error>,
    ReqBody::Data: Send + Unpin,
{
    type Response = http::Response<crate::BoxBody>;

    type Error = crate::Error;

    type Future = crate::BoxFuture<Self::Response, Self::Error>;

    fn get_url(&self) -> Url {
        self.url.clone()
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        self.conn.call(req)
    }
}

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

use std::sync::Once;

use tonic::client::Grpc;
use tonic::transport::Channel;
use tonic::transport::Endpoint;

use crate::common::url::Url;
use crate::protocol::invocation;
use crate::protocol::Invoker;

pub struct GrpcInvoker {
    client: Grpc<Channel>,
    url: Url,
    once: Once,
}

impl GrpcInvoker {
    pub fn new(url: Url) -> GrpcInvoker {
        let endpoint = Endpoint::new(url.to_url()).unwrap();
        let conn = endpoint.connect_lazy();
        Self {
            url,
            client: Grpc::new(conn),
            once: Once::new(),
        }
    }
}

impl Invoker for GrpcInvoker {
    fn is_available(&self) -> bool {
        true
    }

    fn destroy(&self) {
        self.once.call_once(|| println!("destroy..."))
    }

    fn get_url(&self) -> Url {
        self.url.to_owned()
    }

    // 根据req中的数据发起req，由Client发起请求，获取响应
    fn invoke<M1>(&self, req: invocation::Request<M1>) -> invocation::Response<String>
    where
        M1: Send + 'static,
    {
        let (metadata, _) = req.into_parts();

        let resp = invocation::Response::new("string");
        let (_resp_meta, msg) = resp.into_parts();

        invocation::Response::from_parts(metadata, msg.to_string())
    }
}

// impl<T> invocation::Request<T> {

//     pub(crate) fn to_tonic_req(self) -> tonic::Request<T> {
//         tonic::Request::new(self.message)
//     }
// }

impl Clone for GrpcInvoker {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            url: self.url.clone(),
            once: Once::new(),
        }
    }
}

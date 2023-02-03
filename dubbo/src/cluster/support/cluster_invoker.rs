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
use std::sync::Arc;

use http::uri::PathAndQuery;
use http::Request;
use hyper::Body;
use tower_service::Service;

use crate::cluster::loadbalance::types::BoxLoadBalance;
use crate::cluster::loadbalance::LOAD_BALANCE_EXTENSIONS;
use crate::cluster::support::DEFAULT_LOADBALANCE;
use crate::codegen::{Directory, RegistryDirectory, TripleClient};
use crate::common::url::Url;
use crate::invocation::RpcInvocation;
use crate::triple;

#[derive(Debug, Clone)]
pub struct ClusterInvoker {
    directory: Arc<RegistryDirectory>,
    destroyed: bool,
}

pub trait ClusterInvokerSelector {
    /// Select a invoker using loadbalance policy.
    fn select(
        &self,
        invocation: Arc<RpcInvocation>,
        invokers: Arc<Vec<Url>>,
        excluded: Arc<Vec<Url>>,
    ) -> Option<Url>;

    fn do_select(
        &self,
        loadbalance_key: Option<&str>,
        invocation: Arc<RpcInvocation>,
        invokers: Arc<Vec<Url>>,
    ) -> Option<Url>;
}

pub trait ClusterRequestBuilder<T>
where
    T: Service<http::Request<hyper::Body>, Response = http::Response<crate::BoxBody>>,
    T::Error: Into<crate::Error>,
{
    fn build_req(
        &self,
        triple_client: &TripleClient<T>,
        path: http::uri::PathAndQuery,
        invocation: Arc<RpcInvocation>,
        body: hyper::Body,
    ) -> http::Request<hyper::Body>;
}

impl ClusterInvoker {
    pub fn with_directory(registry_directory: RegistryDirectory) -> Self {
        ClusterInvoker {
            directory: Arc::new(registry_directory),
            destroyed: false,
        }
    }

    pub fn directory(&self) -> Arc<RegistryDirectory> {
        self.directory.clone()
    }

    pub fn init_loadbalance(&self, loadbalance_key: &str) -> &BoxLoadBalance {
        if LOAD_BALANCE_EXTENSIONS.contains_key(loadbalance_key) {
            LOAD_BALANCE_EXTENSIONS.get(loadbalance_key).unwrap()
        } else {
            println!(
                "loadbalance {} not found, use default loadbalance {}",
                loadbalance_key, DEFAULT_LOADBALANCE
            );
            LOAD_BALANCE_EXTENSIONS.get(DEFAULT_LOADBALANCE).unwrap()
        }
    }

    pub fn is_available(&self, invocation: Arc<RpcInvocation>) -> bool {
        !self.destroyed() && !self.directory.list(invocation).is_empty()
    }

    pub fn destroyed(&self) -> bool {
        self.destroyed
    }
}

impl ClusterInvokerSelector for ClusterInvoker {
    fn select(
        &self,
        invocation: Arc<RpcInvocation>,
        invokers: Arc<Vec<Url>>,
        _excluded: Arc<Vec<Url>>,
    ) -> Option<Url> {
        if invokers.is_empty() {
            return None;
        }
        let instance_count = invokers.len();
        return if instance_count == 1 {
            Some(invokers.as_ref().first()?.clone())
        } else {
            let loadbalance = Some(DEFAULT_LOADBALANCE);
            self.do_select(loadbalance, invocation, invokers)
        };
    }

    /// picking instance invoker url from registry directory
    fn do_select(
        &self,
        loadbalance_key: Option<&str>,
        invocation: Arc<RpcInvocation>,
        invokers: Arc<Vec<Url>>,
    ) -> Option<Url> {
        let loadbalance = self.init_loadbalance(loadbalance_key.unwrap_or(DEFAULT_LOADBALANCE));
        loadbalance.select(invokers, None, invocation)
    }
}

impl<T> ClusterRequestBuilder<T> for ClusterInvoker
where
    T: Service<http::Request<hyper::Body>, Response = http::Response<crate::BoxBody>>,
    T::Error: Into<crate::Error>,
{
    fn build_req(
        &self,
        triple_client: &triple::client::triple::TripleClient<T>,
        path: PathAndQuery,
        invocation: Arc<RpcInvocation>,
        body: Body,
    ) -> Request<Body> {
        let invokers = self.directory.list(invocation.clone());
        let invoker_url = self
            .select(invocation, invokers, Arc::new(Vec::new()))
            .expect("no valid provider");
        let http_uri =
            http::Uri::from_str(&format!("http://{}:{}/", invoker_url.ip, invoker_url.port))
                .unwrap();
        TripleClient::new_map_request(triple_client, http_uri, path, body)
    }
}

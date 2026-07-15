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

use std::time::Duration;

use bytes::Bytes;
use dubbo::{
    codegen::{ClientBuilder, Request, RpcInvocation, TripleClient},
    invocation::Metadata,
    status::{Code, Status},
};

/// A stable raw Triple client facade for embedding Dubbo Rust in other runtimes.
///
/// This is intentionally byte-oriented. Hosts such as Node.js should own their
/// protobuf message shape and use this facade for Dubbo transport and governance.
pub struct RawTripleClient {
    inner: TripleClient,
}

impl RawTripleClient {
    pub fn from_static(endpoint: &str) -> Self {
        let builder = ClientBuilder::from_static(endpoint).with_direct(true);
        Self {
            inner: TripleClient::new(builder),
        }
    }

    pub async fn unary(
        &mut self,
        request: RawUnaryRequest,
    ) -> Result<RawUnaryResponse, dubbo::status::Status> {
        let path = request.path.parse().map_err(|err| {
            dubbo::status::Status::new(
                dubbo::status::Code::InvalidArgument,
                format!("invalid request path: {err}"),
            )
        })?;
        let invocation = RpcInvocation::default()
            .with_service_unique_name(request.service)
            .with_method_name(request.method);
        let timeout_ms = request.timeout_ms;
        let call = self.inner.raw_unary(
            Request::from_parts(request.metadata.into(), request.body),
            path,
            invocation,
        );
        let response = if let Some(timeout_ms) = timeout_ms {
            tokio::time::timeout(Duration::from_millis(timeout_ms), call)
                .await
                .map_err(|_| {
                    Status::new(
                        Code::DeadlineExceeded,
                        format!("request timed out after {timeout_ms}ms"),
                    )
                })??
        } else {
            call.await?
        };
        let (metadata, body) = response.into_parts();

        Ok(RawUnaryResponse {
            metadata: metadata.into(),
            body,
        })
    }
}

#[derive(Debug, Clone)]
pub struct RawUnaryRequest {
    pub service: String,
    pub method: String,
    pub path: String,
    pub metadata: RawMetadata,
    pub body: Bytes,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct RawUnaryResponse {
    pub metadata: RawMetadata,
    pub body: Bytes,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RawMetadata {
    pub entries: Vec<(String, String)>,
}

impl RawMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.entries.push((key.into(), value.into()));
        self
    }
}

impl From<RawMetadata> for Metadata {
    fn from(value: RawMetadata) -> Self {
        value
            .entries
            .into_iter()
            .fold(Metadata::default(), |metadata, (key, value)| {
                metadata.insert(key, value)
            })
    }
}

impl From<Metadata> for RawMetadata {
    fn from(value: Metadata) -> Self {
        let entries = value
            .into_headers()
            .iter()
            .filter_map(|(key, value)| {
                value
                    .to_str()
                    .ok()
                    .map(|value| (key.to_string(), value.to_string()))
            })
            .collect();

        Self { entries }
    }
}

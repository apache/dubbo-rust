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
    cluster::ClusterStrategy,
    codegen::{ClientBuilder, Request, RpcInvocation, TripleClient},
    invocation::Metadata,
    loadbalancer::LoadBalanceStrategy,
    triple::compression::CompressionEncoding,
    Url,
};
#[cfg(feature = "registry-nacos")]
use dubbo_registry_nacos::NacosRegistry;
#[cfg(feature = "registry-zookeeper")]
use dubbo_registry_zookeeper::ZookeeperRegistry;

pub use dubbo::status::{Code, Status};

/// A stable raw Triple client facade for embedding Dubbo Rust in other runtimes.
///
/// This is intentionally byte-oriented. Hosts such as Node.js should own their
/// protobuf message shape and use this facade for Dubbo transport and governance.
pub struct RawTripleClient {
    inner: TripleClient,
    default_timeout_ms: Option<u64>,
    default_metadata: RawMetadata,
}

impl RawTripleClient {
    pub fn from_static(endpoint: &str) -> Self {
        Self::from_static_endpoints([endpoint]).expect("static endpoint must be a valid Dubbo URL")
    }

    pub fn from_static_endpoints<'a, I>(endpoints: I) -> Result<Self, Status>
    where
        I: IntoIterator<Item = &'a str>,
    {
        Self::from_static_endpoints_with_options(endpoints, RawTripleClientOptions::default())
    }

    pub fn from_static_endpoints_with_options<'a, I>(
        endpoints: I,
        options: RawTripleClientOptions,
    ) -> Result<Self, Status>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let endpoints = endpoints
            .into_iter()
            .map(|endpoint| {
                endpoint.parse::<Url>().map_err(|err| {
                    Status::new(
                        Code::InvalidArgument,
                        format!("invalid static endpoint {endpoint}: {err}"),
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if endpoints.is_empty() {
            return Err(Status::new(
                Code::InvalidArgument,
                "at least one static endpoint is required".to_string(),
            ));
        }
        let builder =
            options.apply(ClientBuilder::from_static_urls(endpoints).with_direct(true))?;
        Ok(Self {
            inner: TripleClient::new(builder),
            default_timeout_ms: options.timeout_ms,
            default_metadata: options.default_metadata,
        })
    }

    pub async fn from_registry(registry_url: &str) -> Result<Self, Status> {
        Self::from_registry_with_options(registry_url, RawTripleClientOptions::default()).await
    }

    pub async fn from_registry_with_options(
        registry_url: &str,
        options: RawTripleClientOptions,
    ) -> Result<Self, Status> {
        let registry_url = registry_url.parse::<Url>().map_err(|err| {
            Status::new(
                Code::InvalidArgument,
                format!("invalid registry URL {registry_url}: {err}"),
            )
        })?;

        register_registry_extension(registry_url.protocol()).await?;

        let builder = options.apply(ClientBuilder::new().with_registry(registry_url))?;
        Ok(Self {
            inner: TripleClient::new(builder),
            default_timeout_ms: options.timeout_ms,
            default_metadata: options.default_metadata,
        })
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
        let timeout_ms = request.timeout_ms.or(self.default_timeout_ms);
        let call = self.inner.raw_unary(
            Request::from_parts(
                self.default_metadata.clone().merge(request.metadata).into(),
                request.body,
            ),
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

#[derive(Debug, Clone, Default)]
pub struct RawTripleClientOptions {
    pub timeout_ms: Option<u64>,
    pub load_balance: Option<String>,
    pub cluster: Option<String>,
    pub failover_retries: Option<u32>,
    pub compression: Option<String>,
    pub default_metadata: RawMetadata,
}

impl RawTripleClientOptions {
    fn apply(&self, builder: ClientBuilder) -> Result<ClientBuilder, Status> {
        let builder = match self.load_balance.as_deref() {
            Some(load_balance) => {
                let strategy = LoadBalanceStrategy::parse(load_balance).ok_or_else(|| {
                    Status::new(
                        Code::InvalidArgument,
                        format!(
                            "unsupported load balance {load_balance}; expected random, round_robin, or p2c"
                        ),
                    )
                })?;
                builder.with_load_balance(strategy)
            }
            None => builder,
        };

        let builder = match self.cluster.as_deref() {
            Some(cluster) => {
                let strategy = ClusterStrategy::parse(cluster).ok_or_else(|| {
                    Status::new(
                        Code::InvalidArgument,
                        format!("unsupported cluster {cluster}; expected failfast or failover"),
                    )
                })?;
                builder.with_cluster(strategy)
            }
            None => builder,
        };

        let builder = match self.failover_retries {
            Some(retries) => {
                let attempts = usize::try_from(retries)
                    .ok()
                    .and_then(|retries| retries.checked_add(1))
                    .ok_or_else(|| {
                        Status::new(
                            Code::InvalidArgument,
                            format!("unsupported failover retries {retries}"),
                        )
                    })?;
                builder.with_failover_attempts(attempts)
            }
            None => builder,
        };

        let builder = match self.compression.as_deref() {
            Some("gzip") => builder.with_compression(Some(CompressionEncoding::Gzip)),
            Some("identity") | Some("none") => builder.with_compression(None),
            Some(compression) => {
                return Err(Status::new(
                    Code::InvalidArgument,
                    format!("unsupported compression {compression}; expected gzip or identity"),
                ));
            }
            None => builder,
        };

        Ok(builder)
    }
}

async fn register_registry_extension(protocol: &str) -> Result<(), Status> {
    match protocol {
        #[cfg(feature = "registry-nacos")]
        "nacos" => dubbo::extension::EXTENSIONS
            .register::<dubbo::extension::registry_extension::RegistryExtension<NacosRegistry>>()
            .await
            .map_err(registry_extension_error),
        #[cfg(feature = "registry-zookeeper")]
        "zookeeper" => dubbo::extension::EXTENSIONS
            .register::<dubbo::extension::registry_extension::RegistryExtension<ZookeeperRegistry>>(
            )
            .await
            .map_err(registry_extension_error),
        protocol => Err(Status::new(
            Code::Unimplemented,
            format!("registry protocol {protocol} is not enabled in dubbo-rs-core"),
        )),
    }
}

#[cfg(any(feature = "registry-nacos", feature = "registry-zookeeper"))]
fn registry_extension_error(err: dubbo::StdError) -> Status {
    Status::new(
        Code::Internal,
        format!("failed to register registry extension: {err}"),
    )
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

    pub fn merge(mut self, other: RawMetadata) -> Self {
        self.entries.extend(other.entries);
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

#[cfg(all(
    test,
    not(any(feature = "registry-nacos", feature = "registry-zookeeper"))
))]
mod tests {
    use super::*;

    #[test]
    fn static_client_rejects_unknown_load_balance() {
        let result = RawTripleClient::from_static_endpoints_with_options(
            ["http://127.0.0.1:50051?interface=example.Echo"],
            RawTripleClientOptions {
                load_balance: Some("least_active".to_string()),
                ..RawTripleClientOptions::default()
            },
        );
        match result {
            Ok(_) => panic!("client should reject unsupported load balance strategy"),
            Err(err) => assert_eq!(err.code(), Code::InvalidArgument),
        }
    }

    #[test]
    fn static_client_rejects_unknown_cluster() {
        let result = RawTripleClient::from_static_endpoints_with_options(
            ["http://127.0.0.1:50051?interface=example.Echo"],
            RawTripleClientOptions {
                cluster: Some("failsafe".to_string()),
                ..RawTripleClientOptions::default()
            },
        );
        match result {
            Ok(_) => panic!("client should reject unsupported cluster strategy"),
            Err(err) => assert_eq!(err.code(), Code::InvalidArgument),
        }
    }

    #[test]
    fn static_client_rejects_unknown_compression() {
        let result = RawTripleClient::from_static_endpoints_with_options(
            ["http://127.0.0.1:50051?interface=example.Echo"],
            RawTripleClientOptions {
                compression: Some("brotli".to_string()),
                ..RawTripleClientOptions::default()
            },
        );
        match result {
            Ok(_) => panic!("client should reject unsupported compression"),
            Err(err) => assert_eq!(err.code(), Code::InvalidArgument),
        }
    }

    #[tokio::test]
    async fn from_registry_reports_disabled_backend() {
        let result = RawTripleClient::from_registry("nacos://127.0.0.1:8848").await;
        match result {
            Ok(_) => panic!("registry client should not be created without registry features"),
            Err(err) => assert_eq!(err.code(), Code::Unimplemented),
        }
    }
}

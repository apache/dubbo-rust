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

use std::{
    convert::Infallible,
    future::Future,
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use bytes::{BufMut, Bytes, BytesMut};
use dubbo::{status::Code, triple::transport::DubboServer, BoxBody};
use dubbo_rs_core::{RawMetadata, RawTripleClient, RawTripleClientOptions, RawUnaryRequest};
use http_body::Body;
use tower_service::Service;

#[derive(Clone)]
struct SlowRawUnaryService {
    delay: Duration,
    response_payload: Bytes,
}

#[derive(Clone)]
struct MetadataEchoService;

#[derive(Clone)]
struct CompressionEchoService;

impl Service<http::Request<hyper::Body>> for SlowRawUnaryService {
    type Response = http::Response<BoxBody>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: http::Request<hyper::Body>) -> Self::Future {
        let delay = self.delay;
        let response_payload = self.response_payload.clone();
        Box::pin(async move {
            tokio::time::sleep(delay).await;
            let body = http_body::combinators::UnsyncBoxBody::new(
                hyper::Body::from(grpc_frame(response_payload)).map_err(|err| {
                    dubbo::status::Status::new(dubbo::status::Code::Internal, err.to_string())
                }),
            );

            Ok(http::Response::builder()
                .status(http::StatusCode::OK)
                .header("content-type", "application/grpc+proto")
                .body(body)
                .unwrap())
        })
    }
}

impl Service<http::Request<hyper::Body>> for MetadataEchoService {
    type Response = http::Response<BoxBody>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<hyper::Body>) -> Self::Future {
        let default_header = req
            .headers()
            .get("x-client-app")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("")
            .to_string();
        let call_header = req
            .headers()
            .get("x-call-id")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("")
            .to_string();

        Box::pin(async move {
            let body = http_body::combinators::UnsyncBoxBody::new(
                hyper::Body::from(grpc_frame(protobuf_string(format!(
                    "{default_header}:{call_header}"
                ))))
                .map_err(|err| {
                    dubbo::status::Status::new(dubbo::status::Code::Internal, err.to_string())
                }),
            );

            Ok(http::Response::builder()
                .status(http::StatusCode::OK)
                .header("content-type", "application/grpc+proto")
                .body(body)
                .unwrap())
        })
    }
}

impl Service<http::Request<hyper::Body>> for CompressionEchoService {
    type Response = http::Response<BoxBody>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<hyper::Body>) -> Self::Future {
        let compression = req
            .headers()
            .get("grpc-encoding")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("identity")
            .to_string();

        Box::pin(async move {
            let body = http_body::combinators::UnsyncBoxBody::new(
                hyper::Body::from(grpc_frame(protobuf_string(compression))).map_err(|err| {
                    dubbo::status::Status::new(dubbo::status::Code::Internal, err.to_string())
                }),
            );

            Ok(http::Response::builder()
                .status(http::StatusCode::OK)
                .header("content-type", "application/grpc+proto")
                .body(body)
                .unwrap())
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn raw_unary_uses_static_endpoint_list_and_maps_timeout() {
    raw_unary_uses_static_endpoint_list().await;
    raw_unary_timeout_returns_deadline_exceeded().await;
}

async fn raw_unary_uses_static_endpoint_list() {
    const SERVICE: &str = "grpc.examples.echo.StaticEndpointEcho";
    let addr = unused_local_addr();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let server_task = spawn_server(
        addr,
        shutdown_rx,
        SERVICE.to_string(),
        Duration::from_millis(0),
    );

    wait_for_server(addr).await;

    let first_endpoint = format!("http://{addr}?interface={SERVICE}&instance=one");
    let second_endpoint = format!("http://{addr}?interface={SERVICE}&instance=two");
    let mut client =
        RawTripleClient::from_static_endpoints([first_endpoint.as_str(), second_endpoint.as_str()])
            .unwrap();
    let response = client
        .unary(RawUnaryRequest {
            service: SERVICE.to_string(),
            method: "UnaryEcho".to_string(),
            path: format!("/{SERVICE}/UnaryEcho"),
            metadata: RawMetadata::new(),
            body: Bytes::from_static(b"\x0a\x08dubbo-js"),
            timeout_ms: Some(10_000),
        })
        .await
        .unwrap();

    assert_eq!(response.body, Bytes::from_static(b"\x0a\x0draw response"));

    let _ = shutdown_tx.send(());
    server_task.await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn raw_unary_applies_default_metadata_and_allows_request_override() {
    const SERVICE: &str = "grpc.examples.echo.DefaultMetadataEcho";
    let addr = unused_local_addr();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let server_task = tokio::spawn(async move {
        DubboServer::new()
            .with_listener("tcp".to_string())
            .add_service(SERVICE.to_string(), MetadataEchoService)
            .serve_with_graceful(addr, async move {
                let _ = shutdown_rx.await;
            })
            .await
            .unwrap();
    });

    wait_for_server(addr).await;

    let endpoint = format!("http://{addr}?interface={SERVICE}");
    let mut client = RawTripleClient::from_static_endpoints_with_options(
        [endpoint.as_str()],
        RawTripleClientOptions {
            default_metadata: RawMetadata::new()
                .insert("x-client-app", "default-app")
                .insert("x-call-id", "default-call"),
            ..RawTripleClientOptions::default()
        },
    )
    .unwrap();
    let response = client
        .unary(RawUnaryRequest {
            service: SERVICE.to_string(),
            method: "UnaryEcho".to_string(),
            path: format!("/{SERVICE}/UnaryEcho"),
            metadata: RawMetadata::new().insert("x-call-id", "request-call"),
            body: Bytes::from_static(b"\x0a\x08dubbo-js"),
            timeout_ms: Some(10_000),
        })
        .await
        .unwrap();

    assert_eq!(response.body, protobuf_string("default-app:request-call"));

    let _ = shutdown_tx.send(());
    server_task.await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn raw_unary_configures_compression() {
    const SERVICE: &str = "grpc.examples.echo.CompressionEcho";
    let addr = unused_local_addr();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let server_task = tokio::spawn(async move {
        DubboServer::new()
            .with_listener("tcp".to_string())
            .add_service(SERVICE.to_string(), CompressionEchoService)
            .serve_with_graceful(addr, async move {
                let _ = shutdown_rx.await;
            })
            .await
            .unwrap();
    });

    wait_for_server(addr).await;

    let endpoint = format!("http://{addr}?interface={SERVICE}");
    let mut gzip_client = RawTripleClient::from_static(&endpoint);
    let gzip_response = gzip_client
        .unary(RawUnaryRequest {
            service: SERVICE.to_string(),
            method: "UnaryEcho".to_string(),
            path: format!("/{SERVICE}/UnaryEcho"),
            metadata: RawMetadata::new(),
            body: Bytes::from_static(b"\x0a\x08dubbo-js"),
            timeout_ms: Some(10_000),
        })
        .await
        .unwrap();

    assert_eq!(gzip_response.body, protobuf_string("gzip"));

    let mut identity_client = RawTripleClient::from_static_endpoints_with_options(
        [endpoint.as_str()],
        RawTripleClientOptions {
            compression: Some("identity".to_string()),
            ..RawTripleClientOptions::default()
        },
    )
    .unwrap();
    let identity_response = identity_client
        .unary(RawUnaryRequest {
            service: SERVICE.to_string(),
            method: "UnaryEcho".to_string(),
            path: format!("/{SERVICE}/UnaryEcho"),
            metadata: RawMetadata::new(),
            body: Bytes::from_static(b"\x0a\x08dubbo-js"),
            timeout_ms: Some(10_000),
        })
        .await
        .unwrap();

    assert_eq!(identity_response.body, protobuf_string("identity"));

    let _ = shutdown_tx.send(());
    server_task.await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn raw_unary_routes_static_endpoints_by_tag() {
    const SERVICE: &str = "grpc.examples.echo.TaggedEndpointEcho";
    let blue_addr = unused_local_addr();
    let green_addr = unused_local_addr();
    let (blue_shutdown_tx, blue_shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let (green_shutdown_tx, green_shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let blue_server_task = spawn_server_with_payload(
        blue_addr,
        blue_shutdown_rx,
        SERVICE.to_string(),
        Duration::from_millis(0),
        Bytes::from_static(b"\x0a\x04blue"),
    );
    let green_server_task = spawn_server_with_payload(
        green_addr,
        green_shutdown_rx,
        SERVICE.to_string(),
        Duration::from_millis(0),
        Bytes::from_static(b"\x0a\x05green"),
    );

    wait_for_server(blue_addr).await;
    wait_for_server(green_addr).await;

    let blue_endpoint = format!("http://{blue_addr}?interface={SERVICE}&tag=blue");
    let green_endpoint = format!("http://{green_addr}?interface={SERVICE}&tag=green");
    let mut client =
        RawTripleClient::from_static_endpoints([green_endpoint.as_str(), blue_endpoint.as_str()])
            .unwrap();
    let response = client
        .unary(RawUnaryRequest {
            service: SERVICE.to_string(),
            method: "UnaryEcho".to_string(),
            path: format!("/{SERVICE}/UnaryEcho"),
            metadata: RawMetadata::new().insert("dubbo.tag", "blue"),
            body: Bytes::from_static(b"\x0a\x08dubbo-js"),
            timeout_ms: Some(10_000),
        })
        .await
        .unwrap();

    assert_eq!(response.body, Bytes::from_static(b"\x0a\x04blue"));

    let _ = blue_shutdown_tx.send(());
    let _ = green_shutdown_tx.send(());
    blue_server_task.await.unwrap();
    green_server_task.await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn raw_unary_routes_static_endpoints_by_group_and_version() {
    const SERVICE: &str = "grpc.examples.echo.GroupVersionEndpointEcho";
    let blue_addr = unused_local_addr();
    let green_addr = unused_local_addr();
    let (blue_shutdown_tx, blue_shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let (green_shutdown_tx, green_shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let blue_server_task = spawn_server_with_payload(
        blue_addr,
        blue_shutdown_rx,
        SERVICE.to_string(),
        Duration::from_millis(0),
        Bytes::from_static(b"\x0a\x04blue"),
    );
    let green_server_task = spawn_server_with_payload(
        green_addr,
        green_shutdown_rx,
        SERVICE.to_string(),
        Duration::from_millis(0),
        Bytes::from_static(b"\x0a\x05green"),
    );

    wait_for_server(blue_addr).await;
    wait_for_server(green_addr).await;

    let blue_endpoint = format!("http://{blue_addr}?interface={SERVICE}&group=blue&version=1.0.0");
    let green_endpoint =
        format!("http://{green_addr}?interface={SERVICE}&group=green&version=1.0.0");
    let mut client =
        RawTripleClient::from_static_endpoints([green_endpoint.as_str(), blue_endpoint.as_str()])
            .unwrap();
    let response = client
        .unary(RawUnaryRequest {
            service: SERVICE.to_string(),
            method: "UnaryEcho".to_string(),
            path: format!("/{SERVICE}/UnaryEcho"),
            metadata: RawMetadata::new()
                .insert("tri-service-group", "blue")
                .insert("tri-service-version", "1.0.0"),
            body: Bytes::from_static(b"\x0a\x08dubbo-js"),
            timeout_ms: Some(10_000),
        })
        .await
        .unwrap();

    assert_eq!(response.body, Bytes::from_static(b"\x0a\x04blue"));

    let _ = blue_shutdown_tx.send(());
    let _ = green_shutdown_tx.send(());
    blue_server_task.await.unwrap();
    green_server_task.await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn raw_unary_errors_when_routing_metadata_matches_no_provider() {
    const SERVICE: &str = "grpc.examples.echo.NoMatchedEndpointEcho";
    let addr = unused_local_addr();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let server_task = spawn_server(
        addr,
        shutdown_rx,
        SERVICE.to_string(),
        Duration::from_millis(0),
    );

    wait_for_server(addr).await;

    let endpoint = format!("http://{addr}?interface={SERVICE}&group=green&version=1.0.0");
    let mut client = RawTripleClient::from_static(&endpoint);
    let err = client
        .unary(RawUnaryRequest {
            service: SERVICE.to_string(),
            method: "UnaryEcho".to_string(),
            path: format!("/{SERVICE}/UnaryEcho"),
            metadata: RawMetadata::new()
                .insert("tri-service-group", "blue")
                .insert("tri-service-version", "1.0.0"),
            body: Bytes::from_static(b"\x0a\x08dubbo-js"),
            timeout_ms: Some(10_000),
        })
        .await
        .unwrap_err();

    assert_eq!(err.code(), Code::Unavailable);

    let _ = shutdown_tx.send(());
    server_task.await.unwrap();
}

async fn raw_unary_timeout_returns_deadline_exceeded() {
    const SERVICE: &str = "grpc.examples.echo.TimeoutEcho";
    let addr = unused_local_addr();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let server_task = spawn_server(
        addr,
        shutdown_rx,
        SERVICE.to_string(),
        Duration::from_millis(100),
    );

    wait_for_server(addr).await;

    let endpoint = format!("http://{addr}?interface={SERVICE}");
    let mut client = RawTripleClient::from_static(&endpoint);
    let err = client
        .unary(RawUnaryRequest {
            service: SERVICE.to_string(),
            method: "UnaryEcho".to_string(),
            path: format!("/{SERVICE}/UnaryEcho"),
            metadata: RawMetadata::new(),
            body: Bytes::from_static(b"\x0a\x08dubbo-js"),
            timeout_ms: Some(10),
        })
        .await
        .unwrap_err();

    assert_eq!(err.code(), Code::DeadlineExceeded);

    let _ = shutdown_tx.send(());
    server_task.await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn raw_unary_uses_client_default_timeout() {
    const SERVICE: &str = "grpc.examples.echo.DefaultTimeoutEcho";
    let addr = unused_local_addr();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let server_task = spawn_server(
        addr,
        shutdown_rx,
        SERVICE.to_string(),
        Duration::from_millis(100),
    );

    wait_for_server(addr).await;

    let endpoint = format!("http://{addr}?interface={SERVICE}");
    let mut client = RawTripleClient::from_static_endpoints_with_options(
        [endpoint.as_str()],
        RawTripleClientOptions {
            timeout_ms: Some(10),
            ..RawTripleClientOptions::default()
        },
    )
    .unwrap();
    let err = client
        .unary(RawUnaryRequest {
            service: SERVICE.to_string(),
            method: "UnaryEcho".to_string(),
            path: format!("/{SERVICE}/UnaryEcho"),
            metadata: RawMetadata::new(),
            body: Bytes::from_static(b"\x0a\x08dubbo-js"),
            timeout_ms: None,
        })
        .await
        .unwrap_err();

    assert_eq!(err.code(), Code::DeadlineExceeded);

    let _ = shutdown_tx.send(());
    server_task.await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn raw_unary_request_timeout_overrides_client_default_timeout() {
    const SERVICE: &str = "grpc.examples.echo.OverrideTimeoutEcho";
    let addr = unused_local_addr();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let server_task = spawn_server(
        addr,
        shutdown_rx,
        SERVICE.to_string(),
        Duration::from_millis(20),
    );

    wait_for_server(addr).await;

    let endpoint = format!("http://{addr}?interface={SERVICE}");
    let mut client = RawTripleClient::from_static_endpoints_with_options(
        [endpoint.as_str()],
        RawTripleClientOptions {
            timeout_ms: Some(1),
            ..RawTripleClientOptions::default()
        },
    )
    .unwrap();
    let response = client
        .unary(RawUnaryRequest {
            service: SERVICE.to_string(),
            method: "UnaryEcho".to_string(),
            path: format!("/{SERVICE}/UnaryEcho"),
            metadata: RawMetadata::new(),
            body: Bytes::from_static(b"\x0a\x08dubbo-js"),
            timeout_ms: Some(10_000),
        })
        .await
        .unwrap();

    assert_eq!(response.body, Bytes::from_static(b"\x0a\x0draw response"));

    let _ = shutdown_tx.send(());
    server_task.await.unwrap();
}

fn spawn_server(
    addr: SocketAddr,
    shutdown_rx: tokio::sync::oneshot::Receiver<()>,
    service_name: String,
    delay: Duration,
) -> tokio::task::JoinHandle<()> {
    spawn_server_with_payload(
        addr,
        shutdown_rx,
        service_name,
        delay,
        Bytes::from_static(b"\x0a\x0draw response"),
    )
}

fn spawn_server_with_payload(
    addr: SocketAddr,
    shutdown_rx: tokio::sync::oneshot::Receiver<()>,
    service_name: String,
    delay: Duration,
    response_payload: Bytes,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        DubboServer::new()
            .with_listener("tcp".to_string())
            .add_service(
                service_name,
                SlowRawUnaryService {
                    delay,
                    response_payload,
                },
            )
            .serve_with_graceful(addr, async move {
                let _ = shutdown_rx.await;
            })
            .await
            .unwrap();
    })
}

async fn wait_for_server(addr: SocketAddr) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(1);
    loop {
        if tokio::net::TcpStream::connect(addr).await.is_ok() {
            return;
        }
        if tokio::time::Instant::now() >= deadline {
            panic!("server did not start listening on {addr}");
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

fn grpc_frame(payload: Bytes) -> Bytes {
    let mut frame = BytesMut::with_capacity(5 + payload.len());
    frame.put_u8(0);
    frame.put_u32(payload.len() as u32);
    frame.extend_from_slice(&payload);
    frame.freeze()
}

fn protobuf_string(value: impl AsRef<str>) -> Bytes {
    let value = value.as_ref().as_bytes();
    let mut message = BytesMut::with_capacity(2 + value.len());
    message.put_u8(0x0a);
    message.put_u8(value.len() as u8);
    message.extend_from_slice(value);
    message.freeze()
}

fn unused_local_addr() -> SocketAddr {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
}

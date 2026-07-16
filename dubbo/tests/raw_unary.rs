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
};

use bytes::{BufMut, Bytes, BytesMut};
use dubbo::{
    codegen::{ClientBuilder, Request, RpcInvocation, TripleClient},
    triple::transport::DubboServer,
    BoxBody,
};
use http_body::Body;
use tower_service::Service;

#[derive(Clone)]
struct RawUnaryService;

impl Service<http::Request<hyper::Body>> for RawUnaryService {
    type Response = http::Response<BoxBody>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<hyper::Body>) -> Self::Future {
        Box::pin(async move {
            assert_eq!(req.uri().path(), "/grpc.examples.echo.Echo/UnaryEcho");
            assert_eq!(
                req.headers()
                    .get("tri-service-group")
                    .and_then(|v| v.to_str().ok()),
                Some("test")
            );
            assert_eq!(
                req.headers()
                    .get("tri-service-version")
                    .and_then(|v| v.to_str().ok()),
                Some("1.0.0")
            );

            let response_payload = Bytes::from_static(b"\x0a\x0draw response");
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_unary_round_trips_protobuf_bytes() {
    let addr = unused_local_addr();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let server_task = tokio::spawn(async move {
        DubboServer::new()
            .with_listener("tcp".to_string())
            .add_service("grpc.examples.echo.Echo".to_string(), RawUnaryService)
            .serve_with_graceful(addr, async move {
                let _ = shutdown_rx.await;
            })
            .await
            .unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let endpoint = format!("http://{addr}?interface=grpc.examples.echo.Echo");
    let builder = ClientBuilder::from_static(&endpoint).with_direct(true);
    let mut client = TripleClient::new(builder);
    let path = http::uri::PathAndQuery::from_static("/grpc.examples.echo.Echo/UnaryEcho");
    let invocation = RpcInvocation::default()
        .with_service_unique_name("grpc.examples.echo.Echo".to_string())
        .with_method_name("UnaryEcho".to_string());
    let request = Request::from_parts(
        dubbo::invocation::Metadata::default()
            .insert("tri-service-group".to_string(), "test".to_string())
            .insert("tri-service-version".to_string(), "1.0.0".to_string()),
        Bytes::from_static(b"\x0a\x08dubbo-js"),
    );

    let response = client.raw_unary(request, path, invocation).await.unwrap();

    let (_, response_bytes) = response.into_parts();
    assert_eq!(response_bytes, Bytes::from_static(b"\x0a\x0draw response"));

    let _ = shutdown_tx.send(());
    server_task.await.unwrap();
}

fn grpc_frame(payload: Bytes) -> Bytes {
    let mut frame = BytesMut::with_capacity(5 + payload.len());
    frame.put_u8(0);
    frame.put_u32(payload.len() as u32);
    frame.extend_from_slice(&payload);
    frame.freeze()
}

fn unused_local_addr() -> SocketAddr {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
}

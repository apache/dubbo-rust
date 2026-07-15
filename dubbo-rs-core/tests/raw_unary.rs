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
use dubbo_rs_core::{RawMetadata, RawTripleClient, RawUnaryRequest};
use http_body::Body;
use tower_service::Service;

#[derive(Clone)]
struct SlowRawUnaryService {
    delay: Duration,
}

impl Service<http::Request<hyper::Body>> for SlowRawUnaryService {
    type Response = http::Response<BoxBody>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: http::Request<hyper::Body>) -> Self::Future {
        let delay = self.delay;
        Box::pin(async move {
            tokio::time::sleep(delay).await;
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
async fn raw_unary_timeout_returns_deadline_exceeded() {
    let addr = unused_local_addr();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let server_task = tokio::spawn(async move {
        DubboServer::new()
            .with_listener("tcp".to_string())
            .add_service(
                "grpc.examples.echo.Echo".to_string(),
                SlowRawUnaryService {
                    delay: Duration::from_millis(100),
                },
            )
            .serve_with_graceful(addr, async move {
                let _ = shutdown_rx.await;
            })
            .await
            .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let endpoint = format!("http://{addr}?interface=grpc.examples.echo.Echo");
    let mut client = RawTripleClient::from_static(&endpoint);
    let err = client
        .unary(RawUnaryRequest {
            service: "grpc.examples.echo.Echo".to_string(),
            method: "UnaryEcho".to_string(),
            path: "/grpc.examples.echo.Echo/UnaryEcho".to_string(),
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

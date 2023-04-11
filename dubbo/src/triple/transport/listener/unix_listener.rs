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

use std::{net::SocketAddr, task};

use super::Listener;
use async_trait::async_trait;
use dubbo_logger::tracing;
use futures_core::Stream;
use hyper::server::accept::Accept;
use tokio::net::{UnixListener as tokioUnixListener, UnixStream};

pub struct UnixListener {
    inner: tokioUnixListener,
    path: String,
}

impl UnixListener {
    pub async fn bind(addr: SocketAddr) -> std::io::Result<UnixListener> {
        let listener = tokioUnixListener::bind(format!("{}", addr.to_string()))?;

        Ok(UnixListener {
            inner: listener,
            path: addr.to_string(),
        })
    }
}

#[async_trait]
impl Listener for UnixListener {
    type Conn = UnixStream;

    async fn accept(&self) -> std::io::Result<(Self::Conn, SocketAddr)> {
        let (unix_stream, _unix_addr) = self.inner.accept().await?;
        let addr: SocketAddr = self.path.parse().unwrap();
        Ok((unix_stream, addr))
    }
}

impl Stream for UnixListener {
    type Item = UnixStream;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.inner.poll_accept(cx).map(|res| match res {
            Ok(data) => Some(data.0),
            Err(err) => {
                tracing::error!("UnixListener poll_next Error: {:?}", err);
                None
            }
        })
    }
}

impl Accept for UnixListener {
    type Conn = UnixStream;

    type Error = crate::Error;

    fn poll_accept(
        self: std::pin::Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> std::task::Poll<Option<Result<Self::Conn, Self::Error>>> {
        self.inner.poll_accept(cx).map(|res| match res {
            Ok(data) => Some(Ok(data.0)),
            Err(err) => {
                tracing::error!("UnixListener poll_accept Error: {:?}", err);
                None
            }
        })
    }
}

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
    fmt, io,
    task::{Context, Poll},
};

use futures::Stream;
use logger::tracing;
use pin_project::pin_project;
use tokio::net::TcpListener;
#[cfg(target_family = "unix")]
use tokio::net::UnixListener;
#[cfg(target_family = "unix")]
use tokio_stream::wrappers::UnixListenerStream;
use tokio_stream::{wrappers::TcpListenerStream, StreamExt};

use super::{conn::Conn, Address};

#[pin_project(project = IncomingProj)]
#[derive(Debug)]
pub enum DefaultIncoming {
    Tcp(#[pin] TcpListenerStream),
    #[cfg(target_family = "unix")]
    Unix(#[pin] UnixListenerStream),
}

#[async_trait::async_trait]
impl MakeIncoming for DefaultIncoming {
    type Incoming = DefaultIncoming;

    async fn make_incoming(self) -> io::Result<Self::Incoming> {
        Ok(self)
    }
}

#[cfg(target_family = "unix")]
impl From<UnixListener> for DefaultIncoming {
    fn from(l: UnixListener) -> Self {
        DefaultIncoming::Unix(UnixListenerStream::new(l))
    }
}

impl From<TcpListener> for DefaultIncoming {
    fn from(l: TcpListener) -> Self {
        DefaultIncoming::Tcp(TcpListenerStream::new(l))
    }
}

#[async_trait::async_trait]
pub trait Incoming: fmt::Debug + Send + 'static {
    async fn accept(&mut self) -> io::Result<Option<Conn>>;
}

#[async_trait::async_trait]
impl Incoming for DefaultIncoming {
    async fn accept(&mut self) -> io::Result<Option<Conn>> {
        if let Some(conn) = self.try_next().await? {
            tracing::trace!("[Net] recv a connection from: {:?}", conn.info.peer_addr);
            Ok(Some(conn))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
pub trait MakeIncoming {
    type Incoming: Incoming;

    async fn make_incoming(self) -> io::Result<Self::Incoming>;
}

#[async_trait::async_trait]
impl MakeIncoming for Address {
    type Incoming = DefaultIncoming;

    async fn make_incoming(self) -> io::Result<Self::Incoming> {
        match self {
            Address::Ip(addr) => TcpListener::bind(addr).await.map(DefaultIncoming::from),
            #[cfg(target_family = "unix")]
            Address::Unix(addr) => UnixListener::bind(addr).map(DefaultIncoming::from),
        }
    }
}

impl Stream for DefaultIncoming {
    type Item = io::Result<Conn>;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project() {
            IncomingProj::Tcp(s) => s.poll_next(cx).map_ok(Conn::from),
            #[cfg(target_family = "unix")]
            IncomingProj::Unix(s) => s.poll_next(cx).map_ok(Conn::from),
        }
    }
}

#[cfg(test)]
mod tests {
    use logger::tracing::debug;
    use tokio::{io::AsyncReadExt, net::TcpListener};
    use tokio_stream::wrappers::TcpListenerStream;

    use crate::{incoming::Incoming, DefaultIncoming, MakeIncoming};

    #[tokio::test]
    async fn test_read_bytes() {
        let listener = TcpListener::bind("127.0.0.1:8858").await.unwrap();
        let incoming = DefaultIncoming::Tcp(TcpListenerStream::new(listener))
            .make_incoming()
            .await
            .unwrap();
        debug!("[Dubbo-Rust] server start at: {:?}", incoming);
        let join_handle = tokio::spawn(async move {
            let mut incoming = incoming;
            match incoming.accept().await.unwrap() {
                Some(mut conn) => {
                    debug!(
                        "[Dubbo-Rust] recv a connection from: {:?}",
                        conn.info.peer_addr
                    );
                    let mut buf = vec![0; 1024];
                    let n = conn.read(&mut buf).await.unwrap();
                    debug!(
                        "[Dubbo-Rust] recv a connection from: {:?}",
                        String::from_utf8(buf[..n].to_vec()).unwrap()
                    );
                }
                None => {
                    debug!("[Dubbo-Rust] recv a connection from: None");
                }
            }
        });
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        drop(join_handle);
    }
}

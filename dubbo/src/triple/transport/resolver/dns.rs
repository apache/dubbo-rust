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
    future::Future,
    net::{SocketAddr, ToSocketAddrs},
    pin::Pin,
    task::Poll,
    vec,
};

use tokio::task::JoinHandle;
use tower_service::Service;

#[derive(Clone, Default)]
pub struct DnsResolver {}

impl Service<String> for DnsResolver {
    type Response = vec::IntoIter<SocketAddr>;

    type Error = std::io::Error;

    type Future = DnsFuture;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, name: String) -> Self::Future {
        let block = tokio::task::spawn_blocking(move || (name, 0).to_socket_addrs());

        DnsFuture { inner: block }
    }
}

pub struct DnsFuture {
    inner: JoinHandle<Result<vec::IntoIter<SocketAddr>, std::io::Error>>,
}

impl Future for DnsFuture {
    type Output = Result<vec::IntoIter<SocketAddr>, std::io::Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|res| match res {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(err)) => Err(err),
            Err(join_err) => {
                if join_err.is_cancelled() {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Interrupted,
                        join_err,
                    ))
                } else {
                    panic!("dnsfuture poll failed: {:?}", join_err)
                }
            }
        })
    }
}

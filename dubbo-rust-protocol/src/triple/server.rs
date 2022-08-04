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

use std::future::Future;
use std::{net::SocketAddr, pin::Pin, task::Poll};

use futures::ready;
use http::{Request as HttpRequest, Response as HttpResponse};
use hyper::server::accept::Accept;
use hyper::server::conn::{AddrIncoming, AddrStream};
use hyper::server::conn::{Connection, Http};
use hyper::Body;
use log::trace;
use pin_project_lite::pin_project;
use tokio::io::{AsyncRead, AsyncWrite};

use super::super::{wrap_future, OneConnection, SrvFut, StdError};
use crate::NamedService;
use tower::util::BoxCloneService;

pin_project! {
    pub struct TripleRpcServer<S> {
        #[pin]
        incoming: AddrIncoming,
        rt_handle: tokio::runtime::Handle,
        service: S,
    }
}

impl<S> TripleRpcServer<S> {
    pub fn new(addr: &SocketAddr, handle: tokio::runtime::Handle, service: S) -> Self
    where
        S: tower::Service<HttpRequest<Body>> + Clone,
    {
        let incoming = AddrIncoming::bind(addr).unwrap();
        Self {
            incoming: incoming,
            rt_handle: handle,
            service,
        }
    }

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Result<AddrStream, std::io::Error>>> {
        let me = self.project();
        me.incoming.poll_accept(cx)
    }
}

impl<S> Future for TripleRpcServer<S>
where
    S: tower::Service<
        HttpRequest<Body>,
        Response = HttpResponse<Body>,
        Error = StdError,
        Future = SrvFut<HttpResponse<Body>, StdError>,
    >,
    S: Clone + Send + 'static + Unpin,
{
    type Output = Result<(), StdError>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        loop {
            let ret = ready!(self.as_mut().poll_next(cx));
            match ret {
                Some(Ok(stream)) => {
                    trace!("Get conn {}", stream.remote_addr());

                    let connection = Http::new()
                        .http2_only(true)
                        .http2_enable_connect_protocol()
                        .serve_connection(stream, self.service.clone());

                    let one_conn = OneConnection { connection };
                    self.rt_handle.spawn(one_conn);
                }
                Some(Err(e)) => return Poll::Ready(Err(e.into())),
                None => return Poll::Ready(Err("option none".into())),
            }
        }
    }
}

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

use std::collections::HashMap;
use std::future::Future;
use std::{net::SocketAddr, pin::Pin, task::Poll};

use futures::ready;
use http::{Request as HttpRequest, Response as HttpResponse};
use hyper::body::HttpBody;
use hyper::server::accept::Accept;
use hyper::server::conn::{AddrIncoming, AddrStream};
use hyper::server::conn::{Connection, Http};
use hyper::Body;
use log::trace;
use pin_project_lite::pin_project;
use tokio::io::{AsyncRead, AsyncWrite};

use super::super::{wrap_future, OneConnection, SrvFut, StdError};
use super::Request as JsonRpcRequest;
use super::Response as JsonRpcResponse;
use crate::NamedService;
use tower::util::BoxCloneService;

pin_project! {
   pub struct JsonRpcServer<S> {
        #[pin]
        incoming: AddrIncoming,
        rt_handle: tokio::runtime::Handle,
        service: S
    }
}

impl<S> JsonRpcServer<S> {
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

impl<S> Future for JsonRpcServer<S>
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
                        .http1_only(true)
                        .http1_keep_alive(true)
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

////////////////////////////////////

#[derive(Clone)]
pub struct JsonRpcService {
    // service: HashMap<String, Box<S>>,
    service: HashMap<String, BoxCloneService<JsonRpcRequest, JsonRpcResponse, StdError>>,
}

pub struct JsonRpcServiceBuilder {
    ident: Option<JsonRpcService>,
}

impl JsonRpcService {
    pub fn builder() -> JsonRpcServiceBuilder {
        JsonRpcServiceBuilder { ident: None }
    }

    pub fn new<S>(service: S) -> Self
    where
        S: tower::Service<
            JsonRpcRequest,
            Response = JsonRpcResponse,
            Error = StdError,
            Future = SrvFut<JsonRpcResponse, StdError>,
        >,
        S: Clone + Send + Sync + 'static,
        S: NamedService,
    {
        let mut mm = HashMap::new();

        mm.insert(S::SERVICE_NAME.to_string(), BoxCloneService::new(service));

        Self { service: mm }
    }

    pub fn add_service<S>(&mut self, service: S) -> Result<(), StdError>
    where
        S: tower::Service<
            JsonRpcRequest,
            Response = JsonRpcResponse,
            Error = StdError,
            Future = SrvFut<JsonRpcResponse, StdError>,
        >,
        S: Clone + Send + Sync + 'static,
        S: NamedService,
    {
        if self.service.contains_key(S::SERVICE_NAME) {
            return Err(format!("dupplicate service name {}", S::SERVICE_NAME).into());
        }

        self.service
            .insert(S::SERVICE_NAME.to_string(), BoxCloneService::new(service));
        Ok(())
    }
}

impl JsonRpcServiceBuilder {
    pub fn add_service<S>(&mut self, service: S) -> Result<(), StdError>
    where
        S: tower::Service<
            JsonRpcRequest,
            Response = JsonRpcResponse,
            Error = StdError,
            Future = SrvFut<JsonRpcResponse, StdError>,
        >,
        S: Clone + Send + Sync + 'static,
        S: NamedService,
    {
        if self.ident.is_none() {
            self.ident.replace(JsonRpcService::new(service));
            return Ok(());
        }

        self.ident.as_mut().unwrap().add_service(service)
    }

    pub fn build(mut self) -> Result<JsonRpcService, StdError> {
        if self.ident.is_none() {
            return Err("nothing build".into());
        }

        Ok(self.ident.take().unwrap())
    }
}

impl tower::Service<HttpRequest<Body>> for JsonRpcService {
    type Response = HttpResponse<Body>;

    type Error = StdError;

    type Future = SrvFut<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: HttpRequest<Body>) -> Self::Future {
        // fetch service
        let service_name = req
            .uri()
            .path()
            .trim_start_matches("/")
            .trim_end_matches("/");

        if !self.service.contains_key(service_name) {
            return wrap_future(async { Ok(HttpResponse::builder().body(Body::empty()).unwrap()) });
        }

        let mut inner_service = self.service.get(service_name).unwrap().clone();
        wrap_future(async move {
            if let Some(data) = req.data().await {
                if let Err(ref e) = data {
                    trace!("Get body error {}", e);
                }
                let data = data?;

                let request = JsonRpcRequest::from_slice(data.to_vec());

                if let Err(ref e) = request {
                    trace!("Serde error {}", e);
                }
                let request = request?;

                let fut = inner_service.call(request);
                let res = fut.await?;

                let response_string = res.to_string()?;

                return Ok(HttpResponse::builder()
                    .body(response_string.into())
                    .unwrap());
            } else {
                trace!("none");
            }

            trace!("get req {:?}", req);
            Ok(HttpResponse::builder().body(Body::empty()).unwrap())
        })
    }
}

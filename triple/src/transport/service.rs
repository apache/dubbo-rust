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
use std::future;
use std::net::SocketAddr;
use std::task::Poll;

use http::{Request, Response};
use hyper::{body::Body, server::conn::AddrStream};
use tokio::time::Duration;
use tower::ServiceExt;
use tower_service::Service;

use crate::BoxBody;
use config::BusinessConfig;
use config::Config;

type BoxService = tower::util::BoxService<Request<Body>, Response<BoxBody>, crate::Error>;
type BoxCloneService = tower::util::BoxCloneService<Request<Body>, Response<BoxBody>, crate::Error>;

#[derive(Default, Clone)]
pub struct DubboServer {
    accept_http2: bool,
    init_stream_window_size: Option<u32>,
    init_connection_window_size: Option<u32>,
    max_concurrent_streams: Option<u32>,
    max_frame_size: Option<u32>,
    http2_keepalive_interval: Option<Duration>,
    http2_keepalive_timeout: Option<Duration>,
    services: HashMap<String, BoxCloneService>,
}

impl DubboServer {
    pub fn with_accpet_http1(self, accept_http2: bool) -> Self {
        Self {
            accept_http2,
            ..self
        }
    }

    pub fn with_init_stream_window_size(self, stream_window: u32) -> Self {
        Self {
            init_stream_window_size: Some(stream_window),
            ..self
        }
    }

    pub fn with_init_connection_window_size(self, connection_window: u32) -> Self {
        Self {
            init_connection_window_size: Some(connection_window),
            ..self
        }
    }
    pub fn with_max_concurrent_streams(self, concurrent_streams: u32) -> Self {
        Self {
            max_concurrent_streams: Some(concurrent_streams),
            ..self
        }
    }
    pub fn with_max_frame_size(self, max_frame_size: u32) -> Self {
        Self {
            max_frame_size: Some(max_frame_size),
            ..self
        }
    }
    pub fn with_http2_keepalive_interval(self, interval: Duration) -> Self {
        Self {
            http2_keepalive_interval: Some(interval),
            ..self
        }
    }

    pub fn with_http2_keepalive_timeout(self, timeout: Duration) -> Self {
        Self {
            http2_keepalive_timeout: Some(timeout),
            ..self
        }
    }
}

impl DubboServer {
    pub fn new() -> Self {
        Self {
            accept_http2: true,
            init_stream_window_size: None,
            init_connection_window_size: None,
            max_concurrent_streams: None,
            http2_keepalive_interval: None,
            http2_keepalive_timeout: None,
            max_frame_size: None,
            services: HashMap::new(),
        }
    }
}

impl DubboServer {
    pub fn add_service<S>(mut self, name: String, service: S) -> Self
    where
        S: Service<Request<Body>, Response = Response<BoxBody>> + Clone + Send + 'static,
        S::Future: Send + 'static,
        S::Error: Into<crate::Error> + Send + 'static,
    {
        self.services
            .insert(name, service.map_err(|err| err.into()).boxed_clone());
        Self { ..self }
    }

    pub async fn serve(self, name: String, addr: SocketAddr) -> Result<(), crate::Error> {
        let svc = MakeSvc {
            inner: self.services.get(&name).unwrap().clone(),
        };

        hyper::Server::bind(&addr)
            .http2_only(self.accept_http2)
            .http2_max_concurrent_streams(self.max_concurrent_streams)
            .http2_initial_connection_window_size(self.init_connection_window_size)
            .http2_initial_stream_window_size(self.init_stream_window_size)
            .http2_keep_alive_interval(self.http2_keepalive_interval)
            .http2_keep_alive_timeout(self.http2_keepalive_timeout.unwrap())
            .http2_max_frame_size(self.max_frame_size)
            .serve(svc)
            .await
            .map_err(|err| println!("Error: {:?}", err))
            .unwrap();

        Ok(())
    }
}

struct MakeSvc<S> {
    inner: S,
}

impl<S> Service<&AddrStream> for MakeSvc<S>
where
    S: Service<Request<Body>, Response = Response<BoxBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<crate::Error> + Send + 'static,
{
    type Response = BoxService;
    type Error = crate::Error;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _conn: &AddrStream) -> Self::Future {
        let svc = self.inner.clone();
        let s = svc.map_err(|err| err.into()).boxed();
        future::ready(Ok(s))
    }
}

impl BusinessConfig for DubboServer {
    fn init() -> Self {
        let conf = config::get_global_config();
        DubboServer::new()
            .with_accpet_http1(conf.bool("dubbo.server.accept_http2".to_string()))
    }

    fn load() -> Result<(), std::convert::Infallible> {
        todo!()
    }
}

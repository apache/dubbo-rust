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

use std::{io, net::SocketAddr, sync::Arc};

use dubbo_logger::tracing;
use futures_core::Future;
use http::{Request, Response};
use hyper::body::Body;
use tokio::time::Duration;
use tokio_rustls::{
    rustls,
    rustls::{Certificate, PrivateKey},
    TlsAcceptor,
};
use tower_service::Service;

use super::{listener::get_listener, router::DubboRouter};
use crate::{triple::transport::io::BoxIO, BoxBody};

#[derive(Default, Clone, Debug)]
pub struct DubboServer {
    accept_http2: bool,
    init_stream_window_size: Option<u32>,
    init_connection_window_size: Option<u32>,
    max_concurrent_streams: Option<u32>,
    max_frame_size: Option<u32>,
    http2_keepalive_interval: Option<Duration>,
    http2_keepalive_timeout: Option<Duration>,
    router: DubboRouter,
    listener: Option<String>,
    certs: Vec<Certificate>,
    keys: Vec<PrivateKey>,
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

    pub fn with_listener(self, name: String) -> Self {
        Self {
            listener: Some(name),
            ..self
        }
    }

    pub fn with_tls(self, certs: Vec<Certificate>, keys: Vec<PrivateKey>) -> Self {
        Self {
            certs: certs,
            keys: keys,
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
            router: DubboRouter::new(),
            listener: None,
            certs: Vec::new(),
            keys: Vec::new(),
        }
    }
}

impl DubboServer {
    pub fn add_service<S>(mut self, name: String, service: S) -> Self
    where
        S: Service<Request<Body>, Response = Response<BoxBody>, Error = std::convert::Infallible>
            + Clone
            + Send
            + 'static,
        S::Future: Send + 'static,
        S::Error: Into<crate::Error> + Send + 'static,
    {
        self.router = self.router.add_service(name, service);
        self
    }

    pub async fn serve(self, addr: SocketAddr) -> Result<(), crate::Error> {
        self.serve_with_graceful(addr, futures_util::future::pending())
            .await
    }

    pub async fn serve_with_graceful(
        self,
        addr: SocketAddr,
        signal: impl Future<Output = ()>,
    ) -> Result<(), crate::Error> {
        let svc = self.router.clone();
        tokio::pin!(signal);

        let http2_keepalive_timeout = self
            .http2_keepalive_timeout
            .unwrap_or_else(|| Duration::new(60, 0));

        let name = match self.listener {
            Some(v) => v,
            None => {
                return Err(Box::new(crate::status::DubboError::new(
                    "listener name is empty".to_string(),
                )));
            }
        };

        let acceptor: Option<TlsAcceptor>;
        if self.certs.len() != 0 && !self.keys.len() != 0 {
            let mut keys = self.keys;

            let config = rustls::ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_single_cert(self.certs, keys.remove(0))
                .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

            acceptor = Some(TlsAcceptor::from(Arc::new(config)));
        } else {
            acceptor = None;
        }

        let listener = match get_listener(name, addr).await {
            Ok(v) => v,
            Err(err) => return Err(err),
        };

        loop {
            tokio::select! {
                _ = &mut signal => {
                    tracing::info!("graceful shutdown");
                    break
                }
                res = listener.accept() => {
                    match res {
                        Ok(conn) => {
                            let (io, local_addr) = conn;
                            let b :BoxIO;

                            if !acceptor.is_none() {
                                b = BoxIO::new(acceptor.as_ref().unwrap().clone().accept(io).await?);
                            } else {
                                b = io;
                            }

                            tracing::debug!("hyper serve, local address: {:?}", local_addr);
                            let c = hyper::server::conn::Http::new()
                                .http2_only(self.accept_http2)
                                .http2_max_concurrent_streams(self.max_concurrent_streams)
                                .http2_initial_connection_window_size(self.init_connection_window_size)
                                .http2_initial_stream_window_size(self.init_stream_window_size)
                                .http2_keep_alive_interval(self.http2_keepalive_interval)
                                .http2_keep_alive_timeout(http2_keepalive_timeout)
                                .http2_max_frame_size(self.max_frame_size)
                                .serve_connection(b,svc.clone()).with_upgrades();

                            tokio::spawn(c);
                        },
                        Err(err) => tracing::error!("hyper serve, err: {:?}", err),
                    }
                }
            }
        }

        drop(listener);

        Ok(())
    }
}

// impl BusinessConfig for DubboServer {
//     fn init() -> Self {
//         let conf = config::get_global_config();
//         DubboServer::new().with_accpet_http1(conf.bool("dubbo.server.accept_http2".to_string()))
//     }

//     fn load() -> Result<(), std::convert::Infallible> {
//         todo!()
//     }
// }

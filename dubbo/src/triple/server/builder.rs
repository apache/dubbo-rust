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
    net::{SocketAddr, ToSocketAddrs},
    path::Path,
    str::FromStr,
};

use dubbo_base::Url;
use dubbo_logger::tracing;
use http::{Request, Response, Uri};
use hyper::body::Body;
use tokio_rustls::rustls::{Certificate, PrivateKey};
use tower_service::Service;

use crate::{triple::transport::DubboServer, utils, BoxBody};

#[derive(Clone, Default, Debug)]
pub struct ServerBuilder {
    pub listener: String,
    pub addr: Option<SocketAddr>,
    pub certs: Vec<Certificate>,
    pub keys: Vec<PrivateKey>,
    pub service_names: Vec<String>,
    server: DubboServer,
}

impl ServerBuilder {
    pub fn new() -> Self {
        ServerBuilder::default()
    }

    pub fn with_listener(self, listener: String) -> ServerBuilder {
        Self { listener, ..self }
    }

    pub fn with_tls(self, certs: &str, keys: &str) -> ServerBuilder {
        Self {
            certs: match utils::tls::load_certs(Path::new(certs)) {
                Ok(v) => v,
                Err(err) => {
                    tracing::error!("error loading tls certs {:?}", err);
                    Vec::new()
                }
            },
            keys: match utils::tls::load_keys(Path::new(keys)) {
                Ok(v) => v,
                Err(err) => {
                    tracing::error!("error loading tls keys {:?}", err);
                    Vec::new()
                }
            },
            ..self
        }
    }

    pub fn with_addr(self, addr: &'static str) -> ServerBuilder {
        Self {
            addr: addr.to_socket_addrs().unwrap().next(),
            ..self
        }
    }

    pub fn with_service_names(self, service_names: Vec<String>) -> ServerBuilder {
        Self {
            service_names,
            ..self
        }
    }

    pub fn build(self) -> Self {
        let mut server = self.server.with_listener(self.listener.clone());

        {
            if self.certs.len() != 0 && self.keys.len() != 0 {
                server = server.with_tls(self.certs.clone(), self.keys.clone());
            }
        }

        {
            let lock = crate::protocol::triple::TRIPLE_SERVICES.read().unwrap();
            for name in self.service_names.iter() {
                if lock.get(name).is_none() {
                    tracing::warn!("service ({}) not register", name);
                    continue;
                }
                let svc = lock.get(name).unwrap();

                server = server.add_service(name.clone(), svc.clone());
            }
        }

        {}
        Self { server, ..self }
    }

    pub fn add_service<S>(self, name: String, service: S) -> Self
    where
        S: Service<Request<Body>, Response = Response<BoxBody>, Error = std::convert::Infallible>
            + Clone
            + Send
            + 'static,
        S::Future: Send + 'static,
        S::Error: Into<crate::Error> + Send + 'static,
    {
        Self {
            server: self.server.add_service(name, service),
            ..self
        }
    }

    pub async fn serve(self) -> Result<(), crate::Error> {
        tracing::info!("server starting. addr: {:?}", self.addr.unwrap());
        self.server.serve(self.addr.unwrap()).await
    }
}

impl From<Url> for ServerBuilder {
    fn from(u: Url) -> Self {
        let uri = match http::Uri::from_str(&u.raw_url_string()) {
            Ok(v) => v,
            Err(err) => {
                tracing::error!("http uri parse error: {}, url: {:?}", err, &u);
                Uri::default()
            }
        };

        let authority = uri.authority().unwrap();

        Self {
            listener: u.get_param("listener").unwrap_or("tcp".to_string()),
            addr: authority.to_string().to_socket_addrs().unwrap().next(),
            service_names: vec![u.service_name],
            server: DubboServer::default(),
            certs: Vec::new(),
            keys: Vec::new(),
        }
    }
}

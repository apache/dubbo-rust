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
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    str::FromStr,
    sync::Arc,
};

use dubbo_logger::tracing;
use http::Uri;
use hyper::client::connect::dns::Name;
use rustls_native_certs::load_native_certs;
use tokio::net::TcpStream;
use tokio_rustls::{
    client::TlsStream,
    rustls::{self, OwnedTrustAnchor},
    TlsConnector as TlsConnectorTokio,
};
use tower_service::Service;

use crate::triple::transport::resolver::{dns::DnsResolver, Resolve};

#[derive(Clone, Default)]
pub struct HttpsConnector<R = DnsResolver> {
    resolver: R,
}

impl HttpsConnector {
    pub fn new() -> Self {
        Self {
            resolver: DnsResolver::default(),
        }
    }
}

impl<R> HttpsConnector<R> {
    pub fn new_with_resolver(resolver: R) -> HttpsConnector<R> {
        Self { resolver }
    }
}

impl<R> Service<Uri> for HttpsConnector<R>
where
    R: Resolve + Clone + Send + Sync + 'static,
    R::Future: Send,
{
    type Response = TlsStream<TcpStream>;

    type Error = crate::Error;

    type Future = crate::BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.resolver.poll_ready(cx).map_err(|err| err.into())
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        let mut inner = self.clone();

        Box::pin(async move { inner.call_async(uri).await })
    }
}

impl<R> HttpsConnector<R>
where
    R: Resolve + Send + Sync + 'static,
{
    async fn call_async(&mut self, uri: Uri) -> Result<TlsStream<TcpStream>, crate::Error> {
        let host = uri.host().unwrap();
        let port = uri.port_u16().unwrap();

        let addr = if let Ok(addr) = host.parse::<Ipv4Addr>() {
            tracing::info!("host is ip address: {:?}", host);
            SocketAddr::V4(SocketAddrV4::new(addr, port))
        } else {
            tracing::info!("host is dns: {:?}", host);
            let addrs = self
                .resolver
                .resolve(Name::from_str(host).unwrap())
                .await
                .map_err(|err| err.into())?;
            let addrs: Vec<SocketAddr> = addrs
                .map(|mut addr| {
                    addr.set_port(port);
                    addr
                })
                .collect();
            addrs[0]
        };

        let mut root_store = rustls::RootCertStore::empty();

        root_store.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }));
        for cert in load_native_certs()? {
            root_store.add(&rustls::Certificate(cert.0))?;
        }

        let config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let connector = TlsConnectorTokio::from(Arc::new(config));

        let stream = TcpStream::connect(&addr).await?;
        let domain = rustls::ServerName::try_from(host).map_err(|err| {
            crate::status::Status::new(crate::status::Code::Internal, err.to_string())
        })?;
        let stream = connector.connect(domain, stream).await?;

        Ok(stream)
    }
}

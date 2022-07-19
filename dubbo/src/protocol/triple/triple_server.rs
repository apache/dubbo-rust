use std::{net::ToSocketAddrs, str::FromStr};

use triple::transport::DubboServer;

#[derive(Default, Clone)]
pub struct TripleServer {
    s: DubboServer,
    name: String,
}

impl TripleServer {
    pub fn new(name: String) -> TripleServer {
        Self {
            name,
            ..Default::default()
        }
    }

    pub async fn serve(mut self, url: String) {
        {
            let lock = super::TRIPLE_SERVICES.read().unwrap();
            let svc = lock.get(&self.name).unwrap();

            self.s = self.s.add_service(self.name.clone(), svc.clone());
        }

        let uri = http::Uri::from_str(&url.clone()).unwrap();
        let server = self.s.clone();

        server
            .serve(
                self.name,
                uri.authority()
                    .unwrap()
                    .to_string()
                    .to_socket_addrs()
                    .unwrap()
                    .next()
                    .unwrap(),
            )
            .await
            .unwrap();
    }
}

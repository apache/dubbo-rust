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

use std::{net::ToSocketAddrs, str::FromStr};

use triple::transport::DubboServer;

#[derive(Default, Clone)]
pub struct TripleServer {
    s: DubboServer,
    service_names: Vec<String>,
}

impl TripleServer {
    pub fn new(names: Vec<String>) -> TripleServer {
        Self {
            service_names: names,
            s: DubboServer::new(),
        }
    }

    pub async fn serve(mut self, url: String) {
        {
            let lock = super::TRIPLE_SERVICES.read().unwrap();
            for name in self.service_names.iter() {
                let svc = lock.get(name).unwrap();

                self.s = self.s.add_service(name.clone(), svc.clone());
            }
        }

        let uri = http::Uri::from_str(&url.clone()).unwrap();

        self.s
            .with_listener("tcp".to_string())
            .serve(
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

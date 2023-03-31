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

use dubbo_base::Url;
use std::{
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        Arc,
    },
    time,
    time::Duration,
};

use crate::{
    error::ClientError,
    exchange::{Request, Response},
};

pub struct BoxedClient(Arc<dyn Client>);

pub trait Client: Sync + Send {
    fn connect(&self, url: Url) -> Result<(), ClientError>;
    fn request(&self, request: Request, timeout: Duration) -> Result<Response, ClientError>;
    fn close(&mut self) -> Result<(), ClientError>;
    fn is_available(&self) -> bool;
}

pub struct ExchangeClient {
    connection_timeout: Duration, // timeout when connecting to server
    address: String,              // listening ip:port
    client: Option<BoxedClient>,  // dealing with the transports
    init: AtomicBool,             // whether the client is initialized
    active: AtomicI32,            // the number of active service bind to this client
}

impl ExchangeClient {
    pub fn new(url: Url, client: BoxedClient, connection_timeout: Duration) -> Self {
        ExchangeClient {
            connection_timeout,
            address: url.get_ip_port(),
            client: None,
            init: AtomicBool::new(false),
            active: AtomicI32::new(0),
        }
    }
}

impl Client for ExchangeClient {
    fn connect(&self, url: Url) -> Result<(), ClientError> {
        if self.init.load(Ordering::SeqCst) {
            return Ok(());
        }
        Ok(())
    }

    fn request(&self, request: Request, timeout: Duration) -> Result<Response, ClientError> {
        todo!()
    }

    fn close(&mut self) -> Result<(), ClientError> {
        self.init.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn is_available(&self) -> bool {
        let client = self.client.as_ref().unwrap();
        client.0.is_available()
    }
}

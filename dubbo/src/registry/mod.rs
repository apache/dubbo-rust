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

use crate::{extension, extension::registry_extension::proxy::RegistryProxy, StdError, Url};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower_service::Service;

pub mod integration;
pub mod protocol;
pub mod registry;

#[derive(Clone)]
pub struct MkRegistryService {
    registry_url: Url,
}

impl MkRegistryService {
    pub fn new(registry_url: Url) -> Self {
        Self { registry_url }
    }
}

impl Service<()> for MkRegistryService {
    type Response = RegistryProxy;
    type Error = StdError;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: ()) -> Self::Future {
        let fut = extension::EXTENSIONS.load_registry(self.registry_url.clone());
        Box::pin(fut)
    }
}

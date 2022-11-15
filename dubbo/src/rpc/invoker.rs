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
use std::rc::Rc;
use std::sync::Arc;
use crate::common::url::Url;
use crate::invocation::Invocation;


#[derive(Debug, Clone)]
pub struct Invoker {
    registry_url: Url,
    is_available: bool,
    url: Url,
}

impl Invoker {

    pub fn new(registry_url: Url, is_available: bool, url: Url) -> Invoker {
        Invoker {
            registry_url,
            is_available,
            url,
        }
    }
    pub fn invoke(&self, invocation: Box<dyn Invocation>) {}

    pub fn url(&self) -> Url {
        self.url.clone()
    }

    pub fn registry_url(&self) -> Url {
        self.url.clone()
    }
}

/// Container of many invoker
pub type InvokerList = Vec<Arc<Invoker>>;

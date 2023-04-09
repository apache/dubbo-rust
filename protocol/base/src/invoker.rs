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
use crate::{
    invocation::BoxInvocation,
    output::{BoxOutput, RPCOutput},
};
use dubbo_base::{Node, Url};
use std::{
    fmt::{Display, Formatter},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

pub struct BaseInvoker {
    url: Arc<Url>,
    available: AtomicBool,
    destroyed: AtomicBool,
}

pub trait Invoker {
    type Output;
    fn invoke(&self, invocation: BoxInvocation) -> Self::Output;
}

impl Invoker for BaseInvoker {
    type Output = BoxOutput<String>;
    fn invoke(&self, _invocation: BoxInvocation) -> Self::Output {
        Arc::new(RPCOutput::default())
    }
}

impl Node for BaseInvoker {
    fn get_url(&self) -> Arc<Url> {
        self.url.clone()
    }

    fn is_available(&self) -> bool {
        self.available.load(Ordering::SeqCst)
    }

    fn destroy(&self) {
        self.destroyed.store(true, Ordering::SeqCst);
        self.available.store(false, Ordering::SeqCst)
    }
    fn is_destroyed(&self) -> bool {
        self.destroyed.load(Ordering::SeqCst)
    }
}

impl Display for BaseInvoker {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Invoker")
            .field("protocol", &self.url.scheme)
            .field("host", &self.url.ip)
            .field("path", &self.url.location)
            .finish()
    }
}

impl BaseInvoker {
    pub fn new(url: Url) -> Self {
        Self {
            url: Arc::new(url),
            available: AtomicBool::new(true),
            destroyed: AtomicBool::new(false),
        }
    }
}

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
pub mod condition;
pub mod manager;
pub mod nacos_config_center;
pub mod router_chain;
pub mod tag;
pub mod utils;

use crate::invocation::RpcInvocation;
use dubbo_base::Url;
use std::{fmt::Debug, sync::Arc};

pub trait Router: Debug {
    fn route(&self, invokers: Vec<Url>, url: Url, invocation: Arc<RpcInvocation>) -> Vec<Url>;
}

pub type BoxRouter = Box<dyn Router + Sync + Send>;

#[derive(Debug, Default, Clone)]
pub struct MockRouter {}

impl Router for MockRouter {
    fn route(&self, invokers: Vec<Url>, _url: Url, _invocation: Arc<RpcInvocation>) -> Vec<Url> {
        invokers
    }
}

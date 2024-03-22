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

use crate::cluster::router::tag::tag_router::{TagRouter, TagRouterInner};
use dubbo_config::router::TagRouterConfig;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Default)]
pub struct TagRouterManager {
    pub tag_router: Arc<RwLock<TagRouterInner>>,
}

impl TagRouterManager {
    pub fn get_router(&self, _service_name: &String) -> Option<TagRouter> {
        Some(TagRouter {
            inner: self.tag_router.clone(),
        })
    }

    pub fn update(&mut self, config: TagRouterConfig) {
        self.tag_router.write().unwrap().parse_config(config);
    }
}

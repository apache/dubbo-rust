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
    cluster::router::{utils::to_original_map, Router},
    codegen::RpcInvocation,
};
use crate::Url;
use dubbo_config::router::TagRouterConfig;
use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone, Default)]
pub struct TagRouterInner {
    pub tag_rules: HashMap<String, HashMap<String, String>>,
    pub force: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TagRouter {
    pub(crate) inner: Arc<RwLock<TagRouterInner>>,
}
impl Router for TagRouter {
    fn route(&self, invokers: Vec<Url>, url: Url, invocation: Arc<RpcInvocation>) -> Vec<Url> {
        return self.inner.read().unwrap().route(invokers, url, invocation);
    }
}

impl TagRouterInner {
    pub fn parse_config(&mut self, config: TagRouterConfig) {
        self.tag_rules = HashMap::new();
        self.force = config.force;
        self.enabled = config.enabled;
        for tag in &config.tags {
            let mut tags = HashMap::new();
            for rule in &tag.matches {
                tags.insert(rule.key.clone(), rule.value.clone());
            }
            self.tag_rules.insert(tag.name.clone(), tags);
        }
    }

    pub fn match_tag(&self, params: HashMap<String, String>) -> Option<String> {
        let mut tag_result = None;
        for (tag, tag_rules) in &self.tag_rules {
            for (key, value) in tag_rules {
                match params.get(key.as_str()) {
                    None => {}
                    Some(val) => {
                        if val == value {
                            tag_result = Some(tag.clone())
                        }
                    }
                }
            }
        }
        tag_result
    }

    pub fn route(&self, invokers: Vec<Url>, url: Url, _invocation: Arc<RpcInvocation>) -> Vec<Url> {
        if !self.enabled {
            return invokers;
        };
        let self_param = to_original_map(url);
        let invocation_tag = self.match_tag(self_param);
        let mut invokers_result = Vec::new();
        let mut invokers_no_tag = Vec::new();
        for invoker in &invokers {
            let invoker_param = to_original_map(invoker.clone());
            let invoker_tag = self.match_tag(invoker_param);
            if invoker_tag == None {
                invokers_no_tag.push(invoker.clone());
            }
            if invoker_tag == invocation_tag {
                invokers_result.push(invoker.clone());
            }
        }
        if invokers_result.is_empty() {
            if !self.force {
                return invokers_no_tag;
            }
        }
        invokers_result
    }
}

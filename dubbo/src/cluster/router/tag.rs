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

use super::Router;
use crate::{
    context::{Context, RpcContext},
    invocation::Invocation,
};
use dubbo_base::{
    constants::{FORCE_USE_TAG, TAG_KEY},
    Url,
};
use std::sync::Arc;

const DEFAULT_PRIORITY: i32 = 100;
const ROUTER_URL_STR: &str = "tag://0.0.0.0:0/?runtime=true";

pub struct TagRouter {
    url: Url,
    priority: i32,
}

impl TagRouter {
    pub fn new() -> Self {
        let url = Url::from_url(ROUTER_URL_STR).unwrap();
        let priority = extract_priority(&url);
        TagRouter { url, priority }
    }
}

impl Router for TagRouter {
    fn get_url(&self) -> Option<&Url> {
        Some(&self.url)
    }

    fn get_priority(&self) -> i32 {
        self.priority
    }

    fn route(&self, invokers: &Vec<Url>, _: Url, _: Arc<dyn Invocation>) -> Vec<Url> {
        let tag_maybe = tag_from_ctx(TAG_KEY);
        let res = match tag_maybe.as_deref() {
            // TAG_KEY unspecified
            None | Some("") => vec![],
            // otherwise match tags from invokers with tag from context
            _ => invokers
                .iter()
                .filter(|x| x.get_param(TAG_KEY) == tag_maybe)
                .map(|x| x.clone())
                .collect(),
        };

        if res.is_empty() {
            // If TAG_KEY unspecified or no invoker be selected, downgrade to normal invokers
            let force_maybe = tag_from_ctx(FORCE_USE_TAG);
            if let None | Some("false") = force_maybe.as_deref() {
                // Only forceTag = true force match, otherwise downgrade
                return invokers
                    .iter()
                    .filter(|x| match x.get_param(TAG_KEY).as_deref() {
                        None | Some("") => true,
                        _ => false,
                    })
                    .map(|x| x.clone())
                    .collect();
            }
        }
        res
    }
}

/* ----------------------------- Private Utils ----------------------------- */

fn extract_priority(url: &Url) -> i32 {
    url.get_param("priority")
        .and_then(|x| x.parse::<i32>().ok())
        .unwrap_or(DEFAULT_PRIORITY)
}

fn tag_from_ctx(tag: &str) -> Option<String> {
    let attachments = RpcContext::get_attachments()?;
    let lock_guard = attachments.lock().ok()?;
    lock_guard.get(tag).map(|x| x.as_str().unwrap().to_string())
}

/* ------------------------------ Unit Tests ------------------------------- */

#[cfg(test)]
mod tests {
    use crate::{
        cluster::router::{tag::TAG_KEY, Router, TagRouter},
        codegen::RpcInvocation,
        context::{Context, RpcContext},
    };
    use dubbo_base::Url;
    use lazy_static::lazy_static;
    use std::sync::Arc;

    const RED_INVOKER_URL: &str = "dubbo://10.20.3.1:20880/com.foo.BarService?dubbo.tag=red";
    const YELLOW_INVOKER_URL: &str = "dubbo://10.20.3.1:20880/com.foo.BarService?dubbo.tag=yellow";
    const BLUE_INVOKER_URL: &str = "dubbo://10.20.3.1:20880/com.foo.BarService?dubbo.tag=blue";
    const DEFAULT_INVOKER_URL: &str = "dubbo://10.20.3.4:20880/com.foo.BarService";

    lazy_static! {
        static ref R_INVOKER: Url = Url::from_url(RED_INVOKER_URL).unwrap();
        static ref Y_INVOKER: Url = Url::from_url(YELLOW_INVOKER_URL).unwrap();
        static ref B_INVOKER: Url = Url::from_url(BLUE_INVOKER_URL).unwrap();
        static ref D_INVOKER: Url = Url::from_url(DEFAULT_INVOKER_URL).unwrap();
    }

    fn set_tag_key(key: &str) {
        let attachments = RpcContext::get_attachments().unwrap();
        let mut lock_guard = attachments.lock().unwrap();
        lock_guard.insert(TAG_KEY.to_string(), key.parse().unwrap());
    }

    fn reset_ctx() {
        let attachments = RpcContext::get_attachments().unwrap();
        let mut lock_guard = attachments.lock().unwrap();
        lock_guard.clear();
    }

    fn test_route_match_tag() {
        set_tag_key("\"red\"");

        let invokers = vec![
            R_INVOKER.clone(),
            Y_INVOKER.clone(),
            B_INVOKER.clone(),
            D_INVOKER.clone(),
        ];

        let tag_router = TagRouter::new();
        let filtered_invokers = tag_router.route(
            &invokers,
            Url::from_url("consumer://127.0.0.1:0/com.foo.BarService").unwrap(),
            Arc::new(RpcInvocation::default()),
        );

        assert!(filtered_invokers.contains(&R_INVOKER));
        assert!(!filtered_invokers.contains(&Y_INVOKER));
        assert!(!filtered_invokers.contains(&B_INVOKER));
        assert!(!filtered_invokers.contains(&D_INVOKER));

        reset_ctx();
    }

    fn test_route_match_default() {
        set_tag_key("\"\"");

        let invokers = vec![
            R_INVOKER.clone(),
            Y_INVOKER.clone(),
            B_INVOKER.clone(),
            D_INVOKER.clone(),
        ];

        let tag_router = TagRouter::new();
        let filtered_invokers = tag_router.route(
            &invokers,
            Url::from_url("consumer://127.0.0.1:0/com.foo.BarService").unwrap(),
            Arc::new(RpcInvocation::default()),
        );

        assert!(filtered_invokers.contains(&D_INVOKER));
        assert!(!filtered_invokers.contains(&R_INVOKER));
        assert!(!filtered_invokers.contains(&Y_INVOKER));
        assert!(!filtered_invokers.contains(&B_INVOKER));

        reset_ctx();
    }

    fn test_route_request_with_tag_should_downgrade() {
        set_tag_key("\"black\"");

        let invokers = vec![
            R_INVOKER.clone(),
            Y_INVOKER.clone(),
            B_INVOKER.clone(),
            D_INVOKER.clone(),
        ];

        let tag_router = TagRouter::new();
        let filtered_invokers = tag_router.route(
            &invokers,
            Url::from_url("consumer://127.0.0.1:0/com.foo.BarService").unwrap(),
            Arc::new(RpcInvocation::default()),
        );

        assert!(filtered_invokers.contains(&D_INVOKER));
        assert!(!filtered_invokers.contains(&R_INVOKER));
        assert!(!filtered_invokers.contains(&Y_INVOKER));
        assert!(!filtered_invokers.contains(&B_INVOKER));

        reset_ctx();
    }

    fn test_route_request_without_tag_should_not_downgrade() {
        set_tag_key("\"\"");

        let invokers = vec![R_INVOKER.clone(), Y_INVOKER.clone(), B_INVOKER.clone()];
        let tag_router = TagRouter::new();
        let filtered_invokers = tag_router.route(
            &invokers,
            Url::from_url("consumer://127.0.0.1:0/com.foo.BarService").unwrap(),
            Arc::new(RpcInvocation::default()),
        );

        assert!(filtered_invokers.is_empty());

        reset_ctx();
    }

    #[test]
    fn test_tag_router() {
        // run in serial, pass
        // unable to pass with spawning tasks in tokio
        test_route_match_tag();
        test_route_match_default();
        test_route_request_with_tag_should_downgrade();
        test_route_request_without_tag_should_not_downgrade();
    }
}

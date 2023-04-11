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

use std::time::{SystemTime, UNIX_EPOCH};

use dubbo_logger::tracing;
use serde_json::Value;

use crate::{
    codegen::Request,
    context::{Context, RpcContext},
    filter::{TIMEOUT_COUNTDOWN, TIMEOUT_DEFAULT, TRI_TIMEOUT_DEADLINE_IN_NANOS},
    status::Status,
};

use super::Filter;

#[derive(Clone)]
pub struct ContextFilter {}

impl Filter for ContextFilter {
    fn call(&mut self, req: Request<()>) -> Result<Request<()>, Status> {
        let headers = &mut req.metadata.into_headers();

        let timeout = headers.get(TIMEOUT_COUNTDOWN);

        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let mut dead_line_in_nanos = 0_u128;

        if let Some(t) = timeout {
            let timeout: u128 = t.to_str().unwrap().parse().unwrap();
            if timeout > 0_u128 {
                dead_line_in_nanos = time + timeout * 1000000;
            }
        } else {
            let timeout: u128 = TIMEOUT_DEFAULT * 1000000;
            dead_line_in_nanos = time + timeout;
        }

        tracing::debug!(
            "ContextFilter tri-timeout-deadline-in-nanos : {}",
            dead_line_in_nanos
        );
        if let Some(at) = RpcContext::get_attachments() {
            let mut attachments = at.lock().unwrap();
            attachments.insert(
                String::from(TRI_TIMEOUT_DEADLINE_IN_NANOS),
                Value::from(dead_line_in_nanos.to_string()),
            );
        }

        Ok(req)
    }
}

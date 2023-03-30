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
use dubbo_logger::tracing;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    codegen::Request,
    context::{Context, RpcContext},
    filter::TRI_TIMEOUT_DEADLINE_IN_NANOS,
    status::{Code, Status},
};

use super::Filter;

#[derive(Clone)]
pub struct TimeoutFilter {}

/// timeout count
/// 1. ContextFilter 初始化 timeout 时间，初始化后将 tri-timeout-deadline-in-nanos 放入 context 中
/// 2. TimeoutFilter read context tri-timeout-deadline-in-nanos
/// 3. 响应时计算 tri-timeout-deadline-in-nanos - current_nanos <= 0
///
impl Filter for TimeoutFilter {
    fn call(&mut self, req: Request<()>) -> Result<Request<()>, Status> {
        if let Some(attachments) = RpcContext::get_attachments() {
            let current_nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();

            let attachments = attachments.lock().unwrap();
            let tri_timeout_deadline_in_nanos =
                attachments.get(TRI_TIMEOUT_DEADLINE_IN_NANOS).unwrap();
            let tri_timeout_deadline_in_nanos: u128 = tri_timeout_deadline_in_nanos
                .as_str()
                .unwrap()
                .parse()
                .unwrap();

            tracing::debug!(
                "TimeoutFilter tri-timeout-deadline-in-nanos : {}, current-nanos:{}",
                tri_timeout_deadline_in_nanos,
                current_nanos
            );
            if tri_timeout_deadline_in_nanos - current_nanos <= 0 {
                return Err(Status::new(Code::DeadlineExceeded, String::from("Timeout")));
            }
        }

        Ok(req)
    }
}

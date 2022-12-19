use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    codegen::Request,
    context::{Context, RpcContext},
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
                attachments.get("tri-timeout-deadline-in-nanos").unwrap();
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

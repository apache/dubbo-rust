use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::{
    codegen::Request,
    context::{Context, RpcContext},
    status::Status,
};

use super::Filter;

#[derive(Clone)]
pub struct ContextFilter {}

impl Filter for ContextFilter {
    fn call(&mut self, req: Request<()>) -> Result<Request<()>, Status> {
        let headers = &mut req.metadata.into_headers();

        let timeout = headers.get("timeout-countdown");

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
            // TODO default timeout
            let timeout: u128 = 1000_u128 * 1000000;
            dead_line_in_nanos = time + timeout;
        }

        tracing::debug!(
            "ContextFilter tri-timeout-deadline-in-nanos : {}",
            dead_line_in_nanos
        );
        if let Some(at) = RpcContext::get_attachments() {
            let mut attachments = at.lock().unwrap();
            attachments.insert(
                String::from("tri-timeout-deadline-in-nanos"),
                Value::from(dead_line_in_nanos.to_string()),
            );
        }

        Ok(req)
    }
}

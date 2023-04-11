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
use std::{any::Any, fmt::Debug, sync::Arc};

use dashmap::DashMap;

use crate::error::InvokerError;

pub type AttachmentsMap = DashMap<String, String>;

pub struct RPCOutput<R: Any + Debug> {
    error: Option<Arc<InvokerError>>,
    result: Option<Arc<R>>,
    attachments: AttachmentsMap,
}

// role of Output is same to Result, because of preload std::result::Result
pub trait Output<R> {
    fn set_error(&mut self, error: Arc<InvokerError>);
    fn error(&self) -> Option<Arc<InvokerError>>;
    fn set(&mut self, result: R);
    fn get(&self) -> Option<Arc<R>>;
    fn set_attachments(&mut self, attachments: AttachmentsMap);
    fn add_attachment(&mut self, key: &str, value: &str);
    fn get_attachment_or_default(&self, key: &str, default_value: &str) -> String;
}

pub type BoxOutput<R> = Arc<dyn Output<R> + Send + Sync + 'static>;

impl<R> Default for RPCOutput<R>
where
    R: Any + Debug,
{
    fn default() -> Self {
        RPCOutput {
            error: None,
            result: None,
            attachments: AttachmentsMap::new(),
        }
    }
}

impl<R> Output<R> for RPCOutput<R>
where
    R: Any + Debug,
{
    fn set_error(&mut self, error: Arc<InvokerError>) {
        self.error = Some(error);
    }

    fn error(&self) -> Option<Arc<InvokerError>> {
        self.error.clone()
    }

    fn set(&mut self, result: R)
    where
        R: Any + Debug,
    {
        self.result = Some(Arc::new(result))
    }

    fn get(&self) -> Option<Arc<R>> {
        self.result.clone()
    }

    fn set_attachments(&mut self, attachments: AttachmentsMap) {
        self.attachments = attachments;
    }

    fn add_attachment(&mut self, key: &str, value: &str) {
        self.attachments.insert(key.to_string(), value.to_string());
    }

    fn get_attachment_or_default(&self, key: &str, default_value: &str) -> String {
        self.attachments
            .contains_key(key)
            .then(|| self.attachments.get(key).unwrap().clone())
            .unwrap_or(default_value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result() {
        let mut result: RPCOutput<String> = RPCOutput::default();
        result.set("r".to_string());
        assert_eq!(result.get().unwrap().as_str(), "r");
        result.add_attachment("hello", "world");
        let string = result.get_attachment_or_default("hello", "test");
        println!("{}", string);
    }
}

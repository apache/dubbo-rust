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

use core::cell::RefCell;
use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

///
/// ```rust
/// use std::collections::HashMap;
/// use std::sync::Arc;
///
/// let mut map = HashMap::<String, SafetyValue>::new();
///  map.insert("key1".into(), Arc::new("data-1"));
///
///  // get a typed value from SafetyValue
///  let value = map
///      .get("key1")
///      .and_then(|f| f.downcast_ref::<String>())
///      .unwrap();
///
/// assert_eq!(value, "data-1");
/// ```
type SafetyValue = Arc<dyn Any + Sync + Send>;

thread_local! {
    static SERVICE_CONTEXT: RefCell<RpcContext> = RefCell::new(RpcContext::default());
}

///
/// All environment information of during the current call will put into the context
/// on the filter composing process,and all configuration information will convert the parameters of URL instance.
///
/// RpcContext is a temporary status recorder of [thread_local],when accept RPC request or send RPC request,
///  The RpcContext will be changed.Such as: A call B and B call C.
/// On B machine,before B call C,the RpcContext will record the information of A call B.
/// After B call C,the RpcContext record the information of B call C
///
#[derive(Clone, Default)]
pub struct RpcContext {
    pub attachments: HashMap<String, SafetyValue>,
    // TODO
}

impl RpcContext {
    pub fn current() -> Self {
        get_current(|ctx| ctx.clone())
    }

    pub fn clear(&mut self) {
        self.attachments.clear();
    }
}

fn get_current<F: FnMut(&RpcContext) -> T, T>(mut f: F) -> T {
    SERVICE_CONTEXT.try_with(|ctx| f(&ctx.borrow())).unwrap()
}

impl fmt::Debug for RpcContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Context")
            .field("attachments", &self.attachments)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn context_with_thread_local() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .max_blocking_threads(2)
            .enable_all()
            .build()
            .unwrap();

        let mut handles = Vec::with_capacity(10);

        for i in 0..10 {
            handles.push(rt.spawn(async move {
                let mut attachments = RpcContext::current().attachments;
                attachments.insert("key1".into(), Arc::new(format!("data-{i}")));

                if i == 10 {
                    attachments.insert("key2".into(), Arc::new(2));
                    assert_eq!(attachments.len(), 2);
                } else {
                    assert_eq!(attachments.len(), 1);
                }
            }));
        }

        sleep(Duration::from_millis(500));

        for handle in handles {
            rt.block_on(handle).unwrap();
        }
        assert_eq!(RpcContext::current().attachments.len(), 0);
    }
}

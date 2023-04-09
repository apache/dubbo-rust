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

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};

use dubbo_logger::tracing;
use serde_json::Value;
use state::Container;

pub static APPLICATION_CONTEXT: Container![Send + Sync] = <Container![Send + Sync]>::new();

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
pub struct RpcContext {}

pub trait Context {
    fn get_attachments() -> Option<Arc<Mutex<HashMap<String, Value>>>>;
}

impl Context for RpcContext {
    fn get_attachments() -> Option<Arc<Mutex<HashMap<String, Value>>>> {
        let local = APPLICATION_CONTEXT.try_get_local::<Arc<Mutex<HashMap<String, Value>>>>();

        tracing::debug!("{:?} - {:?}", thread::current().id(), local);

        match local {
            Some(attachment) => Some(attachment.clone()),
            None => {
                let attachment = HashMap::<String, Value>::new();
                let mutex = Arc::new(Mutex::new(attachment));
                let mutex_clone = Arc::clone(&mutex);
                APPLICATION_CONTEXT.set_local(move || {
                    return Arc::clone(&mutex_clone);
                });
                Some(Arc::clone(&mutex))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::time;

    use super::*;
    use std::time::Duration;

    #[test]
    fn context_with_thread_local() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .max_blocking_threads(2)
            .enable_all()
            .build()
            .unwrap();

        let mut handles = Vec::with_capacity(10);

        for i in 0..=10 {
            handles.push(rt.spawn(async move {
                if let Some(attachments) = RpcContext::get_attachments() {
                    let mut attachments = attachments.lock().unwrap();
                    attachments.insert("key1".into(), Value::from(format!("data-{i}")));

                    assert!(attachments.len() > 0);
                };

                time::sleep(Duration::from_millis(1000)).await;

                if let Some(attachments) = RpcContext::get_attachments() {
                    let attachments = attachments.lock().unwrap();
                    assert!(attachments.len() > 0);
                };
            }));
        }

        for handle in handles {
            rt.block_on(handle).unwrap();
        }
    }
}

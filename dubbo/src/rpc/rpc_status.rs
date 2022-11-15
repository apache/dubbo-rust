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

use std::collections::HashMap;
use std::convert::identity;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::common::url::Url;

type SharedDb = Arc<RwLock<HashMap<String, RpcStatus>>>;

lazy_static!(
    static ref SERVICE_STATISTICS: SharedDb = Arc::new(RwLock::new(HashMap::new()));
    static ref METHOD_STATISTICS: SharedDb = Arc::new(RwLock::new(HashMap::new()));
);

pub type RpcStatus = Arc<Mutex<RpcStatusInner>>;

#[derive(Debug)]
pub struct RpcStatusInner {
    values: HashMap<String, String>,
    active: AtomicUsize,
    total: AtomicUsize,
    failed: AtomicUsize,
    total_elapsed: AtomicUsize,
    failed_elapsed: AtomicUsize,
    max_elapsed: AtomicUsize,
    failed_max_elapsed: AtomicUsize,
    succeeded_max_elapsed: AtomicUsize,
}

impl RpcStatusInner {
    pub fn new() -> RpcStatusInner {
        RpcStatusInner {
            values: HashMap::new(),
            active: AtomicUsize::new(0),
            total: AtomicUsize::new(0),
            failed: AtomicUsize::new(0),
            total_elapsed: AtomicUsize::new(0),
            failed_elapsed: AtomicUsize::new(0),
            max_elapsed: AtomicUsize::new(0),
            failed_max_elapsed: AtomicUsize::new(0),
            succeeded_max_elapsed: AtomicUsize::new(0),
        }
    }
}

pub trait RpcStatusInnerOps {
    fn succeeded_max_elapsed(&self) -> usize;
    fn total(&self) -> usize;
    fn active(&self) -> usize;
    fn failed(&self) -> usize;
    fn failed_max_elapsed(&self) -> usize;
    fn failed_elapsed(&self) -> usize;
    fn max_elapsed(&self) -> usize;
    fn total_elapsed(&self) -> usize;
    fn set(&mut self, k: &str, v: &str);
    fn get(&self, k: &str) -> Option<String>;
}

pub trait RpcStatusOps {
    fn get_status(url: Url) -> RpcStatus;
    fn get_method_status(url: Url, method_name: &str) -> RpcStatus;
    fn remove_status(url: Url);
    fn remove_method_status(url: Url, method_name: &str);
}

impl RpcStatusOps for RpcStatus {
    fn get_status(url: Url) -> RpcStatus {
        let mut read_guard = SERVICE_STATISTICS.read().unwrap();
        let identity = url.to_identity_string();
        let mut option = read_guard.get(identity.as_str());
        match option {
            None => {
                drop(read_guard);
                let mut write_guard = SERVICE_STATISTICS.write().unwrap();
                write_guard.insert(identity.to_string(), RpcStatus::new(Mutex::new(RpcStatusInner::new())));
                Arc::clone(write_guard.get(identity.as_str()).unwrap())
            }
            Some(v) => Arc::clone(&v)
        }
    }

    fn get_method_status(url: Url, method_name: &str) -> RpcStatus {
        let mut read_guard = SERVICE_STATISTICS.read().unwrap();
        let identity = url.to_identity_string_with_method(method_name);
        let mut option = read_guard.get(identity.as_str());
        match option {
            None => {
                drop(read_guard);
                let mut write_guard = SERVICE_STATISTICS.write().unwrap();
                write_guard.insert(identity.to_string(), RpcStatus::new(Mutex::new(RpcStatusInner::new())));
                Arc::clone(write_guard.get(identity.as_str()).unwrap())
            }
            Some(v) => Arc::clone(v)
        }
    }

    fn remove_status(url: Url) {
        let mut mutex_guard = SERVICE_STATISTICS.write().unwrap();
        let identity = url.to_identity_string();
        mutex_guard.remove(identity.as_str());
    }

    fn remove_method_status(url: Url, method_name: &str) {
        let mut mutex_guard = SERVICE_STATISTICS.write().unwrap();
        let identity = url.to_identity_string_with_method(method_name);
        mutex_guard.remove(identity.as_str());
    }
}

/// operations of RpcStatusInnerOps
impl RpcStatusInnerOps for RpcStatus {
    fn succeeded_max_elapsed(&self) -> usize {
        self.lock().unwrap().succeeded_max_elapsed.load(Ordering::SeqCst)
    }

    fn total(&self) -> usize {
        self.lock().unwrap().total.load(Ordering::SeqCst)
    }

    fn active(&self) -> usize {
        self.lock().unwrap().active.load(Ordering::SeqCst)
    }

    fn failed(&self) -> usize {
        self.lock().unwrap().failed.load(Ordering::SeqCst)
    }

    fn failed_max_elapsed(&self) -> usize {
        self.lock().unwrap().failed_max_elapsed.load(Ordering::SeqCst)
    }

    fn failed_elapsed(&self) -> usize {
        self.lock().unwrap().failed_elapsed.load(Ordering::SeqCst)
    }

    fn max_elapsed(&self) -> usize {
        self.lock().unwrap().max_elapsed.load(Ordering::SeqCst)
    }

    fn total_elapsed(&self) -> usize {
        self.lock().unwrap().total_elapsed.load(Ordering::SeqCst)
    }

    fn set(&mut self, k: &str, v: &str) {
        self.lock().unwrap().values.insert(k.to_string(), v.to_string());
    }

    fn get(&self, k: &str) -> Option<String> {
        match self.lock().unwrap().values.get(k) {
            None => None,
            Some(v) => Some(v.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::{Borrow, BorrowMut};
    use std::ops::Deref;

    use axum::routing::get;

    use crate::common::url::Url;

    use super::*;

    #[test]
    fn test_get_status() {
        let url = Url::from_url("triple://127.0.0.1:8888/helloworld.Greeter");
        let rpc_status = RpcStatus::get_status(url.unwrap());
        println!("{:?}", rpc_status.succeeded_max_elapsed());
    }

    #[test]
    fn test_get_method_status() {
        let url = Url::from_url("triple://127.0.0.1:8888/helloworld.Greeter");
        let rpc_status = RpcStatus::get_method_status(url.unwrap(), "hello");
        println!("{}", rpc_status.total());
    }

    #[test]
    fn test_set_get_values() {
        let url = Url::from_url("triple://127.0.0.1:8888/helloworld.Greeter");
        let mut rpc_status = RpcStatus::get_status(url.unwrap());
        rpc_status.set("q", "a");
        println!("{:?}", rpc_status.get("q").unwrap());
    }
}



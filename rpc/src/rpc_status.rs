use std::any::Any;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use common::dashmap::DashMap;
use common::dashmap::mapref::one::{Ref, RefMut};
use common::lazy_static::lazy_static;
use common::url::Url;
use common::util::url::{to_identity_string_with_url, to_identity_string_with_url_and_method_name};

lazy_static! {
    static ref SERVICE_STATISTICS: Arc<DashMap<String, BoxedRpcStatus>> = Arc::new(DashMap::<String,BoxedRpcStatus>::new());
    static ref METHOD_STATISTICS: Arc<DashMap<String, BoxedRpcStatus>> = Arc::new(DashMap::<String,BoxedRpcStatus>::new());
}


pub type BoxedRpcStatus = Box<RpcStatus>;

#[derive(Debug)]
pub struct RpcStatus {
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

impl RpcStatus {
    pub fn new() -> RpcStatus {
        RpcStatus {
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

    pub fn get_status<'a>(url: Url) -> Ref<'a, String, BoxedRpcStatus> {
        let identity = &to_identity_string_with_url(url);
        let mut option = SERVICE_STATISTICS.get(identity);
        match option {
            None => {
                SERVICE_STATISTICS.insert(identity.to_string(), BoxedRpcStatus::new(RpcStatus::new()));
                SERVICE_STATISTICS.get(identity.as_str()).unwrap()
            }
            Some(v) => v
        }
    }

    pub fn get_method_status<'a>(url: Url, method_name: &str) -> Ref<'a, String, BoxedRpcStatus> {
        let identity = &to_identity_string_with_url_and_method_name(url, method_name);
        let mut option = SERVICE_STATISTICS.get(identity);
        match option {
            None => {
                SERVICE_STATISTICS.insert(identity.to_string(), BoxedRpcStatus::new(RpcStatus::new()));
                SERVICE_STATISTICS.get(identity.as_str()).unwrap()
            }
            Some(v) => v
        }
    }


    pub fn remove_status(url: Url) {
        let identity = &to_identity_string_with_url(url);
        SERVICE_STATISTICS.remove(identity);
    }

    pub fn remove_method_status(url: Url, method_name: &str) {
        let identity = &to_identity_string_with_url_and_method_name(url, method_name);
        METHOD_STATISTICS.remove(identity);
    }

    pub fn succeeded_max_elapsed(&self) -> usize {
        self.succeeded_max_elapsed.load(Ordering::SeqCst)
    }

    pub fn total(&self) -> usize {
        self.total.load(Ordering::SeqCst)
    }

    pub fn active(&self) -> usize {
        self.active.load(Ordering::SeqCst)
    }

    pub fn failed(&self) -> usize {
        self.failed.load(Ordering::SeqCst)
    }

    pub fn failed_max_elapsed(&self) -> usize {
        self.failed_max_elapsed.load(Ordering::SeqCst)
    }

    pub fn failed_elapsed(&self) -> usize {
        self.failed_elapsed.load(Ordering::SeqCst)
    }

    pub fn max_elapsed(&self) -> usize {
        self.max_elapsed.load(Ordering::SeqCst)
    }

    pub fn total_elapsed(&self) -> usize {
        self.total_elapsed.load(Ordering::SeqCst)
    }

    pub fn set(&mut self, k: &str, v: &str) {
        self.values.insert(k.to_string(), v.to_string());
    }

    pub fn get(&self, k: &str) -> Option<String> {
        match self.values.get(k) {
            None => None,
            Some(v) => Some(v.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::BorrowMut;
    use std::ops::DerefMut;
    use std::str::FromStr;
    use super::*;

    #[test]
    fn test_get_status() {
        let url = Url::from_str("https://www.taobao.com").unwrap();
        let option = RpcStatus::get_status(url);
        println!("{:?}", option.succeeded_max_elapsed);
    }

    #[test]
    fn test_get_method_status() {
        let url = Url::from_str("https://www.taobao.com").unwrap();
        let option = RpcStatus::get_method_status(url, "hello");
        option.total.fetch_add(1, Ordering::SeqCst);
        println!("{:?}", option.total());
    }

    #[test]
    fn test_set_get_values(){
        let url = Url::from_str("https://www.taobao.com").unwrap();
        let option = RpcStatus::get_status(url);
        let mut x1 = option.value();
        let x = option.get("a");
        todo!("set value");
        println!("{:?}", x);
    }
}



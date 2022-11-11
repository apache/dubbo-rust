use std::any::Any;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use common::dashmap::DashMap;
use common::dashmap::mapref::one::Ref;
use common::lazy_static::lazy_static;
use common::url::Url;
use common::util::url::{to_identity_string_with_url, to_identity_string_with_url_and_method_name};

lazy_static! {
    static ref SERVICE_STATISTICS: Arc<DashMap<String, BoxedRpcStatus>> = {
        let mut map = Arc::new(DashMap::<String,BoxedRpcStatus>::new());
        map
    };
    static ref METHOD_STATISTICS: Arc<DashMap<String, BoxedRpcStatus>> = {
        let mut map = Arc::new(DashMap::<String,BoxedRpcStatus>::new());
        map
    };
}


pub type BoxedRpcStatus = Box<RpcStatus>;

#[derive(Debug)]
pub struct RpcStatus {
    values: Box<DashMap<String,String>>,
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
            values: Box::new(DashMap::new()),
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
        SERVICE_STATISTICS.remove(to_identity_string_with_url(url).as_str());
    }

    pub fn remove_method_status(url: Url, method_name: &str) {
        METHOD_STATISTICS.remove(to_identity_string_with_url_and_method_name(url, method_name).as_str());
    }
}

#[cfg(test)]
mod tests {
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
        println!("{:?}", option.succeeded_max_elapsed);
    }
}



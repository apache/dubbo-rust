use std::any::Any;
use std::borrow::Borrow;
use std::ops::Deref;
use std::sync::atomic::AtomicUsize;

use lazy_static::lazy_static;

use common::flurry::HashMap;
use common::url::Url;
use common::util::url::{to_identity_string_with_url, to_identity_string_with_url_method_name};

lazy_static! {
    static ref SERVICE_STATISTICS: HashMap<String,  BoxedRpcStatus> = HashMap::new();
    static ref METHOD_STATISTICS: HashMap<String,  BoxedRpcStatus> = HashMap::new();
}


pub type BoxedRpcStatus = Box<RpcStatus>;

#[derive(Debug)]
pub struct RpcStatus {
    values: HashMap<String, Box<dyn Any>>,
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

    pub fn get_status(url: Url) -> &'static BoxedRpcStatus {
        let identity = to_identity_string_with_url(url);
        match SERVICE_STATISTICS.pin().get(&identity) {
            None => {
                let boxed_rpc_status: BoxedRpcStatus = BoxedRpcStatus::from(RpcStatus::new());
                SERVICE_STATISTICS.pin().insert(identity, boxed_rpc_status).unwrap();
                SERVICE_STATISTICS.pin().get(identity.as_str()).unwrap()
            }
            Some(v) => v
        }
    }

    pub fn get_method_status(url: Url, method_name: &str) -> &'static BoxedRpcStatus {
        let identity = to_identity_string_with_url_method_name(url, method_name);
        match SERVICE_STATISTICS.pin().get(&identity) {
            None => {
                let boxed_rpc_status: BoxedRpcStatus = BoxedRpcStatus::from(RpcStatus::new());
                SERVICE_STATISTICS.pin().insert(identity, boxed_rpc_status).unwrap();
                &boxed_rpc_status
            }
            Some(v) => v
        }
    }

    pub fn remove_status(url: Url) {
        SERVICE_STATISTICS.pin().remove(to_identity_string_with_url(url).as_str());
    }

    pub fn remove_method_status(url: Url, method_name: &str) {
        METHOD_STATISTICS.pin().remove(to_identity_string_with_url_method_name(url, method_name).as_str());
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
        println!("{:?}", option);
    }
}



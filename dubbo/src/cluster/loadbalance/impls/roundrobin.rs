use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::cluster::loadbalance::types::{LoadBalance, Metadata};
use crate::codegen::RpcInvocation;
use crate::common::url::Url;

pub struct RoundRobinLoadBalance {
    pub metadata: Metadata,
    pub counter_map: RwLock<HashMap<String,AtomicUsize>>,
}

impl Default for RoundRobinLoadBalance {
    fn default() -> Self {
        RoundRobinLoadBalance {
            metadata: Metadata::new("roundrobin"),
            counter_map: RwLock::new(HashMap::new()),
        }
    }
}

impl Debug for RoundRobinLoadBalance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RoundRobinLoadBalance")
    }
}

impl RoundRobinLoadBalance{
    fn guarantee_counter_key(&self, key: &str) {
        let contained = self.counter_map.try_read().unwrap().contains_key(key);
        if !contained {
            self.counter_map.try_write()
                .unwrap()
                .insert(key.to_string(), AtomicUsize::new(0));
        }
    }
}

impl LoadBalance for RoundRobinLoadBalance {

    fn select(&self, invokers: Arc<Vec<Url>>, _url: Option<Url>, invocation: Arc<RpcInvocation>) -> Option<Url> {
        if invokers.is_empty() {
            return None;
        }
        let fingerprint = invocation.unique_fingerprint();
        self.guarantee_counter_key(fingerprint.as_str());
        let index = self.counter_map.try_read()
            .unwrap()
            .get(fingerprint.as_str())?
            .fetch_add(1,Ordering::SeqCst) % invokers.len();
        Some(invokers[index].clone())
    }

}
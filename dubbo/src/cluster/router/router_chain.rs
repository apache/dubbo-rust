use crate::{cluster::router::BoxRouter, invocation::RpcInvocation};
use dubbo_base::Url;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Default)]
pub struct RouterChain {
    pub routers: HashMap<String, BoxRouter>,
    pub self_url: Url,
}

impl RouterChain {
    pub fn new() -> Self {
        RouterChain {
            routers: HashMap::new(),
            self_url: Url::new(),
        }
    }

    pub fn route(&self, mut invokers: Vec<Url>, invocation: Arc<RpcInvocation>) -> Vec<Url> {
        for (_, value) in self.routers.iter() {
            invokers = value.route(invokers, self.self_url.clone(), invocation.clone())
        }
        invokers
    }

    pub fn add_router(&mut self, key: String, router: BoxRouter) {
        self.routers.insert(key, router);
    }
}

#[test]
fn test() {
    use crate::cluster::router::manager::router_manager::get_global_router_manager;

    let u1 = Url::from_url("tri://127.0.0.1:8888/org.apache.dubbo.sample.tri.Greeter").unwrap();
    let u2 = Url::from_url("tri://127.0.0.1:8889/org.apache.dubbo.sample.tri.Greeter").unwrap();
    let u3 = Url::from_url("tri://127.0.0.1:8800/org.apache.dubbo.sample.tri.Greeter").unwrap();
    let u4 = Url::from_url("tri://127.0.2.1:8880/org.apache.dubbo.sample.tri.Greeter").unwrap();
    let u5 = Url::from_url("tri://127.0.1.1:8882/org.apache.dubbo.sample.tri.Greeter").unwrap();
    let u6 = Url::from_url("tri://213.0.1.1:8888/org.apache.dubbo.sample.tri.Greeter").unwrap();
    let u7 = Url::from_url("tri://169.0.1.1:8887/org.apache.dubbo.sample.tri.Greeter").unwrap();
    let invs = vec![u1, u2, u3, u4, u5, u6, u7];
    let len = invs.len().clone();
    let inv = Arc::new(
        RpcInvocation::default()
            .with_method_name("greet".to_string())
            .with_service_unique_name("org.apache.dubbo.sample.tri.Greeter".to_string()),
    );
    let x = get_global_router_manager()
        .read()
        .unwrap()
        .get_router_chain(inv.get_target_service_unique_name());
    let result = x.route(invs, inv.clone());
    println!("total:{},result:{}", len, result.len().clone());
    dbg!(result);
}

use crate::{
    cluster::router::{
        condition::condition_router::ConditionRouter, tag::tag_router::TagRouter, Router,
    },
    invocation::RpcInvocation,
};
use dubbo_base::Url;
use std::sync::{Arc, RwLock};

#[derive(Debug, Default, Clone)]
pub struct RouterChain {
    pub condition_router: Option<ConditionRouter>,
    pub tag_router: Option<Arc<RwLock<TagRouter>>>,
    pub self_url: Url,
}

impl RouterChain {
    pub fn new() -> Self {
        RouterChain {
            condition_router: None,
            tag_router: None,
            self_url: Url::new(),
        }
    }
    pub fn set_condition_router(&mut self, router: Option<ConditionRouter>) {
        self.condition_router = router;
    }
    pub fn set_tag_router(&mut self, router: Option<Arc<RwLock<TagRouter>>>) {
        self.tag_router = router;
    }
    pub fn route(&self, invokers: Vec<Url>, invocation: Arc<RpcInvocation>) -> Vec<Url> {
        let mut result = invokers.clone();
        match &self.tag_router {
            None => {}
            Some(router) => {
                result =
                    router
                        .read()
                        .unwrap()
                        .route(result, self.self_url.clone(), invocation.clone())
            }
        }
        match &self.condition_router {
            Some(router) => {
                result = router.route(result, self.self_url.clone(), invocation.clone())
            }
            None => {}
        }
        result
    }
}

#[test]
fn test() {
    use crate::cluster::router::manager::router_manager::get_global_router_manager;

    let u1 = Url::from_url("triple://127.0.0.1:8888/org.apache.dubbo.sample.tri.Greeter").unwrap();
    let u2 = Url::from_url("triple://127.0.0.1:8889/org.apache.dubbo.sample.tri.Greeter").unwrap();
    let u3 = Url::from_url("triple://127.0.0.1:8800/org.apache.dubbo.sample.tri.Greeter").unwrap();
    let u4 = Url::from_url("triple://127.0.2.1:880/org.apache.dubbo.sample.tri.Greeter").unwrap();
    let u5 = Url::from_url("triple://127.0.1.1:8888/org.apache.dubbo.sample.tri.Greeter").unwrap();
    let invos = vec![u1, u2, u3, u4, u5];
    let invo = Arc::new(
        RpcInvocation::default()
            .with_method_name("greet".to_string())
            .with_service_unique_name("org.apache.dubbo.sample.tri.Greeter".to_string()),
    );
    let x = get_global_router_manager()
        .read()
        .unwrap()
        .get_router_chain(invo.clone());
    let result = x.route(invos, invo.clone());
    dbg!(result);
}

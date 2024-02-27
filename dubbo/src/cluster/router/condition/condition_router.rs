use crate::{
    cluster::router::{condition::single_router::ConditionSingleRouter, Router},
    codegen::RpcInvocation,
};
use dubbo_base::Url;
use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};

#[derive(Default, Debug, Clone)]
pub struct ConditionRouter {
    //condition router for service scope
    pub service_routers: Option<Arc<RwLock<ConditionSingleRouters>>>,
    //condition router for application  scope
    pub application_routers: Option<Arc<RwLock<ConditionSingleRouters>>>,
}

impl Router for ConditionRouter {
    fn route(&self, mut invokers: Vec<Url>, url: Url, invo: Arc<RpcInvocation>) -> Vec<Url> {
        if let Some(routers) = &self.application_routers {
            for router in &routers.read().unwrap().routers {
                invokers = router.route(invokers, url.clone(), invo.clone());
            }
        }
        if let Some(routers) = &self.service_routers {
            for router in &routers.read().unwrap().routers {
                invokers = router.route(invokers, url.clone(), invo.clone());
            }
        }
        invokers
    }
}

impl ConditionRouter {
    pub fn new(
        service_routers: Option<Arc<RwLock<ConditionSingleRouters>>>,
        application_routers: Option<Arc<RwLock<ConditionSingleRouters>>>,
    ) -> Self {
        Self {
            service_routers,
            application_routers,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConditionSingleRouters {
    pub routers: Vec<ConditionSingleRouter>,
}

impl ConditionSingleRouters {
    pub fn new(routers: Vec<ConditionSingleRouter>) -> Self {
        Self { routers }
    }
    pub fn is_null(&self) -> bool {
        self.routers.is_empty()
    }
}

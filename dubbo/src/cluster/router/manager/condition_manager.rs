use crate::cluster::router::condition::{
    condition_router::{ConditionRouter, ConditionSingleRouters},
    single_router::ConditionSingleRouter,
};
use dubbo_config::router::ConditionRouterConfig;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone, Default)]
pub struct ConditionRouterManager {
    //Application-level routing applies globally, while service-level routing only affects a specific service.
    pub routers_service: HashMap<String, Arc<RwLock<ConditionSingleRouters>>>,
    pub routers_application: Arc<RwLock<ConditionSingleRouters>>,
}

impl ConditionRouterManager {
    pub fn get_router(&self, service_name: String) -> Option<ConditionRouter> {
        let routers_service = self.routers_service.get(&service_name);
        match routers_service {
            Some(routers_service) => {
                if self.routers_application.read().unwrap().is_null() {
                    return Some(ConditionRouter::new(Some(routers_service.clone()), None));
                }
                Some(ConditionRouter::new(
                    Some(routers_service.clone()),
                    Some(self.routers_application.clone()),
                ))
            }
            None => {
                if self.routers_application.read().unwrap().is_null() {
                    return None;
                }
                Some(ConditionRouter::new(
                    None,
                    Some(self.routers_application.clone()),
                ))
            }
        }
    }

    pub fn update(&mut self, config: ConditionRouterConfig) {
        let force = config.force;
        let scope = config.scope;
        let key = config.key;
        let enable = config.enabled;
        let mut routers = Vec::new();
        for condition in config.conditions {
            routers.push(ConditionSingleRouter::new(condition, force, enable));
        }
        match scope.as_str() {
            "application" => {
                self.routers_application.write().unwrap().routers = routers;
            }
            "service" => {
                if let Some(x) = self.routers_service.get(&key) {
                    x.write().unwrap().routers = routers
                } else {
                    self.routers_service.insert(
                        key,
                        Arc::new(RwLock::new(ConditionSingleRouters::new(routers))),
                    );
                }
            }
            _ => {}
        }
    }

    pub fn _parse_rules(&mut self, configs: Vec<ConditionRouterConfig>) {
        for config in configs {
            self.update(config)
        }
    }
}

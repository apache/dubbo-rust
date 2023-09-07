use crate::{
    cluster::router::{
        manager::{condition_manager::ConditionRouterManager, tag_manager::TagRouterManager},
        nacos_config_center::nacos_client::NacosClient,
        router_chain::RouterChain,
    },
    invocation::{Invocation, RpcInvocation},
};
use dubbo_base::Url;
use dubbo_config::{
    get_global_config,
    router::{ConditionRouterConfig, NacosConfig, TagRouterConfig},
};
use dubbo_logger::tracing::{info, trace};
use once_cell::sync::OnceCell;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub static GLOBAL_ROUTER_MANAGER: OnceCell<Arc<RwLock<RouterManager>>> = OnceCell::new();

pub struct RouterManager {
    pub condition_router_manager: ConditionRouterManager,
    pub tag_router_manager: TagRouterManager,
    pub nacos: Option<NacosClient>,
    pub consumer: HashMap<String, Url>,
}

impl RouterManager {
    pub fn get_router_chain(&self, invocation: Arc<RpcInvocation>) -> RouterChain {
        let service = invocation.get_target_service_unique_name();
        let condition_router = self.condition_router_manager.get_router(service.clone());
        let tag_router = self.tag_router_manager.get_router();
        let mut chain = RouterChain::new();
        if let Some(url) = self.consumer.get(service.as_str()) {
            chain.set_condition_router(condition_router);
            chain.set_tag_router(tag_router);
            chain.self_url = url.clone();
        }
        chain
    }

    pub fn notify(&mut self, event: RouterConfigChangeEvent) {
        match event.router_kind.as_str() {
            "condition" => {
                let config: ConditionRouterConfig =
                    serde_yaml::from_str(event.content.as_str()).unwrap();
                self.condition_router_manager.update(config)
            }
            "tag" => {
                let config: TagRouterConfig = serde_yaml::from_str(event.content.as_str()).unwrap();
                self.tag_router_manager.update(config)
            }
            _ => {
                info!("other router change event")
            }
        }
    }

    pub fn init_nacos(&mut self, config: NacosConfig) {
        self.nacos = Some(NacosClient::new_init_client(config));
        self.init_router_managers_for_nacos();
    }

    fn init_router_managers_for_nacos(&mut self) {
        const TAG: &str = "tag";
        const CONDITION: &str = "condition";

        if let Some(tag_config) = self
            .nacos
            .as_ref()
            .and_then(|n| n.get_config("application".to_string(), TAG.to_string(), TAG))
        {
            self.tag_router_manager.init();
            self.tag_router_manager.update(tag_config);
        }

        if let Some(condition_app_config) = self
            .nacos
            .as_ref()
            .and_then(|n| n.get_config("application".to_string(), CONDITION.to_string(), TAG))
        {
            self.condition_router_manager.update(condition_app_config);
        }

        for (service_name, _) in &self.consumer {
            if let Some(condition_config) = self.nacos.as_ref().and_then(|n| {
                n.get_config(service_name.to_string(), CONDITION.to_string(), CONDITION)
            }) {
                self.condition_router_manager.update(condition_config);
            }
        }
    }

    pub fn init(&mut self) {
        let config = get_global_config().routers.clone();
        self.init_consumer_configs();
        if let Some(nacos_config) = &config.nacos {
            self.init_nacos(nacos_config.clone());
        } else {
            trace!("Nacos not configured, using local YAML configuration for routing");
            if let Some(condition_configs) = &config.conditions {
                for condition_config in condition_configs {
                    self.condition_router_manager
                        .update(condition_config.clone());
                }
            } else {
                info!("Unconfigured Condition Router")
            }
            if let Some(tag_config) = &config.tags {
                self.tag_router_manager.init();
                self.tag_router_manager.update(tag_config.clone());
            } else {
                info!("Unconfigured Tag Router")
            }
        }
    }

    fn init_consumer_configs(&mut self) {
        let consumer_configs = get_global_config()
            .routers
            .consumer
            .clone()
            .unwrap_or_else(Vec::new);

        for consumer_config in consumer_configs {
            let service_url = Url::from_url(
                format!("{}/{}", consumer_config.url, consumer_config.service).as_str(),
            )
            .expect("Consumer config error");

            self.consumer.insert(consumer_config.service, service_url);
        }
    }
}

pub fn get_global_router_manager() -> &'static Arc<RwLock<RouterManager>> {
    GLOBAL_ROUTER_MANAGER.get_or_init(|| {
        let mut router_manager = RouterManager {
            condition_router_manager: ConditionRouterManager::default(),
            tag_router_manager: TagRouterManager::default(),
            nacos: None,
            consumer: HashMap::new(),
        };
        router_manager.init();
        return Arc::new(RwLock::new(router_manager));
    })
}

#[derive(Debug, Default, Clone)]
pub struct RouterConfigChangeEvent {
    pub service_name: String,
    pub router_kind: String,
    pub content: String,
}

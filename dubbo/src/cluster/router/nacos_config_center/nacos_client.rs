use crate::cluster::router::manager::router_manager::{
    get_global_router_manager, RouterConfigChangeEvent,
};
use dubbo_config::router::NacosConfig;
use dubbo_logger::{tracing, tracing::info};
use nacos_sdk::api::{
    config::{ConfigChangeListener, ConfigResponse, ConfigService, ConfigServiceBuilder},
    props::ClientProps,
};
use std::sync::{Arc, RwLock};

pub struct NacosClient {
    pub client: Arc<RwLock<dyn ConfigService>>,
}

unsafe impl Send for NacosClient {}

unsafe impl Sync for NacosClient {}

pub struct ConfigChangeListenerImpl;

impl NacosClient {
    pub fn new_init_client(config: NacosConfig) -> Self {
        let server_addr = config.addr;
        let namespace = config.namespace;
        let app = config.app;
        match config.enable_auth {
            None => {
                info!("disable nacos auth!");
                info!("nacos init,addr:{}", server_addr);
                let client = Arc::new(RwLock::new(
                    ConfigServiceBuilder::new(
                        ClientProps::new()
                            .server_addr(server_addr)
                            .namespace(namespace)
                            .app_name(app),
                    )
                    .build()
                    .expect("NacosClient build failed!Please check NacosConfig"),
                ));
                Self { client }
            }
            Some(auth) => {
                info!("enable nacos auth!");
                info!("nacos init,addr:{}", server_addr);
                let client = Arc::new(RwLock::new(
                    ConfigServiceBuilder::new(
                        ClientProps::new()
                            .server_addr(server_addr)
                            .namespace(namespace)
                            .app_name(app)
                            .auth_username(auth.auth_username)
                            .auth_password(auth.auth_password),
                    )
                    // .enable_auth_plugin_http()
                    .build()
                    .expect("NacosClient build failed!Please check NacosConfig"),
                ));
                return Self { client };
            }
        }
    }
    pub fn get_config<T>(&self, data_id: String, group: String, config_type: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let config_resp = self
            .client
            .read()
            .unwrap()
            .get_config(data_id.clone(), group.clone());

        match config_resp {
            Ok(config_resp) => {
                self.add_listener(data_id.clone(), group.clone());
                let string = config_resp.content();
                let result = serde_yaml::from_str(string);
                match result {
                    Ok(config) => {
                        info!(
                            "success to get {}Router config and parse success",
                            config_type
                        );
                        Some(config)
                    }
                    Err(_) => {
                        info!("failed to parse {}Router rule", config_type);
                        None
                    }
                }
            }
            Err(_) => None,
        }
    }
    pub fn add_listener(&self, data_id: String, group: String) {
        let res_listener = self
            .client
            .write()
            .expect("failed to create nacos config listener")
            .add_listener(data_id, group, Arc::new(ConfigChangeListenerImpl {}));
        match res_listener {
            Ok(_) => {
                info!("listening the config success");
            }
            Err(err) => tracing::error!("listen config error {:?}", err),
        }
    }
}

impl ConfigChangeListener for ConfigChangeListenerImpl {
    fn notify(&self, config_resp: ConfigResponse) {
        let content_type = config_resp.content_type();
        let event = RouterConfigChangeEvent {
            service_name: config_resp.data_id().clone(),
            router_kind: config_resp.group().clone(),
            content: config_resp.content().clone(),
        };
        if content_type == "yaml" {
            get_global_router_manager().write().unwrap().notify(event);
        }
        info!("notify config={:?}", config_resp);
    }
}

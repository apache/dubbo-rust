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
        let enable_auth = config.enable_auth;

        let mut props = ClientProps::new()
            .server_addr(server_addr)
            .namespace(namespace)
            .app_name(app);

        if enable_auth.is_some() {
            info!("enable nacos auth!");
        } else {
            info!("disable nacos auth!");
        }

        if let Some(auth) = enable_auth {
            props = props
                .auth_username(auth.auth_username)
                .auth_password(auth.auth_password);
        }

        let client = Arc::new(RwLock::new(
            ConfigServiceBuilder::new(props)
                .build()
                .expect("NacosClient build failed! Please check NacosConfig"),
        ));

        Self { client }
    }

    pub fn get_config<T>(&self, data_id: &str, group: &str, config_type: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let config_resp = self
            .client
            .read()
            .unwrap()
            .get_config(data_id.to_string(), group.to_string());

        match config_resp {
            Ok(config_resp) => {
                self.add_listener(data_id, group);
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

    pub fn add_listener(&self, data_id: &str, group: &str) {
        if let Err(err) = self
            .client
            .write()
            .map_err(|e| format!("failed to create nacos config listener: {}", e))
            .and_then(|client| {
                client
                    .add_listener(
                        data_id.to_string(),
                        group.to_string(),
                        Arc::new(ConfigChangeListenerImpl {}),
                    )
                    .map_err(|e| format!("failed to add nacos config listener: {}", e))
            })
        {
            tracing::error!("{}", err);
        } else {
            info!("listening the config success");
        }
    }
}

impl ConfigChangeListener for ConfigChangeListenerImpl {
    fn notify(&self, config_resp: ConfigResponse) {
        let content_type = config_resp.content_type();
        let event = RouterConfigChangeEvent {
            service_name: config_resp.data_id().to_string(),
            router_kind: config_resp.group().to_string(),
            content: config_resp.content().to_string(),
        };

        if content_type == "yaml" {
            get_global_router_manager().write().unwrap().notify(event);
        }

        info!("notify config: {:?}", config_resp);
    }
}

use std::collections::HashMap;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use crate::{get_global_config};


pub static GLOBAL_CONSUMER_CONFIG: OnceCell<HashMap<String,Consumer>> = OnceCell::new();

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerConfig {
    pub references: Vec<Consumer>,
}

impl ConsumerConfig {
    pub fn get_global_consumer_config(interface:String) -> Option<Consumer> {
        let config=GLOBAL_CONSUMER_CONFIG.get_or_init(|| {
            match get_global_config().consumer.clone() {
                None => {
                    HashMap::new()
                }
                Some(config) => {
                    let mut consumers=HashMap::new();
                    for consumer_config in config.references {
                        consumers.insert(consumer_config.interface.clone(),consumer_config);
                    }
                    consumers
                }
            }
        });
        config.get(interface.as_str()).cloned()
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Consumer {
    pub url: String,
    pub interface: String,
    pub protocol: String,
    pub coding: String,
    pub compress: bool,
}
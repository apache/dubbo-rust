use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct RegistryConfig {
    #[serde(default)]
    pub protocol: String,
    #[serde(default)]
    pub address: String,
}

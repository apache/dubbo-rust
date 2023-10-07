use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct ConditionRouterConfig {
    #[serde(rename = "configVersion")]
    pub config_version: String,
    pub scope: String,
    pub force: bool,
    pub enabled: bool,
    pub key: String,
    pub conditions: Vec<String>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct TagRouterConfig {
    #[serde(rename = "configVersion")]
    pub config_version: String,
    pub force: bool,
    pub enabled: bool,
    pub key: String,
    pub tags: Vec<Tag>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Default, Debug)]
pub struct ConsumerConfig {
    pub service: String,
    pub url: String,
    pub protocol: String,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct Tag {
    pub name: String,
    #[serde(rename = "match")]
    pub matches: Vec<TagMatchRule>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct TagMatchRule {
    pub key: String,
    pub value: String,
}

impl ConditionRouterConfig {
    pub fn new(config: &String) -> Self {
        serde_yaml::from_str(config).expect("parse error")
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct EnableAuth {
    pub auth_username: String,
    pub auth_password: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct NacosConfig {
    pub addr: String,
    pub namespace: String,
    pub app: String,
    pub enable_auth: Option<EnableAuth>,
    pub enable_auth_plugin_http: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct RouterConfig {
    pub consumer: Option<Vec<ConsumerConfig>>,
    pub nacos: Option<NacosConfig>,
    pub conditions: Option<Vec<ConditionRouterConfig>>,
    pub tags: Option<TagRouterConfig>,
}

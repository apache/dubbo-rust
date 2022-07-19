// /// Server 启动的入口。业务侧需要调用该函数进行初始化
// ///
// use std::collections::HashMap;

// use crate::common::url::Url;
// use config::{BusinessConfig, RootConfig};

// pub fn init() {
//     let _root_config = RootConfig::new().load();
//     let service_config = ServiceConfig::default()
//         .group("helloworld".to_string())
//         .serializer("json".to_string())
//         .version("1.0".to_string())
//         .name("echo".to_string());

//     let triple_config = ProtocolConfig::default()
//         .name("triple".to_string())
//         .ip("0.0.0.0".to_string())
//         .port("8888".to_string());

//     let _service_config = service_config.add_protocol_configs(triple_config);
//     // 根据不同的协议，加载不同的配置
//     // 初始化全局的services
//     // let server = DubboServer::init();
//     // let server = server.add_service("echo".to_string(), service);
// }

// #[derive(Default)]
// pub struct ServiceConfig {
//     version: String,
//     group: String,
//     name: String,
//     protocol_names: Vec<String>,
//     registry_names: Vec<String>,
//     serializer: String,
//     protocol_configs: HashMap<String, ProtocolConfig>,
// }

// impl ServiceConfig {
//     pub fn name(self, name: String) -> Self {
//         Self { name, ..self }
//     }

//     pub fn version(self, version: String) -> Self {
//         Self { version, ..self }
//     }

//     pub fn group(self, group: String) -> Self {
//         Self { group, ..self }
//     }

//     pub fn protocol_names(self, protocol_names: Vec<String>) -> Self {
//         Self {
//             protocol_names,
//             ..self
//         }
//     }

//     pub fn serializer(self, serializer: String) -> Self {
//         Self { serializer, ..self }
//     }

//     pub fn add_protocol_configs(mut self, protocol_config: ProtocolConfig) -> Self {
//         self.protocol_configs
//             .insert(protocol_config.name.clone(), protocol_config);
//         Self { ..self }
//     }

//     pub fn get_url(&self) -> Vec<Url> {
//         let mut urls = Vec::new();
//         for (_, conf) in self.protocol_configs.iter() {
//             urls.push(Url {
//                 url: conf.to_owned().to_url(),
//                 service_key: "".to_string(),
//             });
//         }

//         urls
//     }
// }

// #[derive(Default, Debug, Clone)]
// pub struct ProtocolConfig {
//     ip: String,
//     port: String,
//     name: String,
//     params: HashMap<String, String>,
// }

// impl ProtocolConfig {
//     pub fn name(self, name: String) -> Self {
//         Self { name, ..self }
//     }

//     pub fn ip(self, ip: String) -> Self {
//         Self { ip, ..self }
//     }

//     pub fn port(self, port: String) -> Self {
//         Self { port, ..self }
//     }

//     pub fn params(self, params: HashMap<String, String>) -> Self {
//         Self { params, ..self }
//     }

//     pub fn to_url(self) -> String {
//         format!("{}://{}:{}", self.name, self.ip, self.port).to_string()
//     }
// }

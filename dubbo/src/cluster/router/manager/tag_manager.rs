use crate::cluster::router::tag::tag_router::{TagRouter, TagRouterInner};
use dubbo_config::router::TagRouterConfig;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Default)]
pub struct TagRouterManager {
    pub tag_router: Arc<RwLock<TagRouterInner>>,
}

impl TagRouterManager {
    pub fn get_router(&self, _service_name: &String) -> Option<TagRouter> {
        Some(TagRouter {
            inner: self.tag_router.clone(),
        })
    }

    pub fn update(&mut self, config: TagRouterConfig) {
        self.tag_router.write().unwrap().parse_config(config);
    }
}

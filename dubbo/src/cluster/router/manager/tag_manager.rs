use crate::cluster::router::tag::tag_router::TagRouter;
use dubbo_config::router::TagRouterConfig;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Default)]
pub struct TagRouterManager {
    pub tag_router: Option<Arc<RwLock<TagRouter>>>,
}

impl TagRouterManager {
    pub fn init(&mut self) {
        self.tag_router = Some(Arc::new(RwLock::new(TagRouter::default())))
    }

    pub fn get_router(&self) -> Option<Arc<RwLock<TagRouter>>> {
        self.tag_router.clone()
    }

    pub fn update(&mut self, config: TagRouterConfig) {
        self.tag_router
            .as_ref()
            .unwrap()
            .write()
            .unwrap()
            .parse_config(config)
    }
}

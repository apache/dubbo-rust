use crate::loadbalance::Metadata;

#[derive(Debug)]
pub struct LeastActive {
    metadata: Metadata,
}

impl LeastActive {
    pub fn new() -> LeastActive {
        LeastActive {
            metadata: Metadata {
                name: "leastactive".to_string()
            }
        }
    }
}
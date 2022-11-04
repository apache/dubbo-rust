use crate::loadbalance::Metadata;

#[derive(Debug)]
pub struct RoundRobin {
    metadata: Metadata,
}

impl RoundRobin {
    pub fn new() -> RoundRobin {
        RoundRobin {
            metadata: Metadata {
                name: "roundrobin"
            }
        }
    }
}
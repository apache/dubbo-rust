use crate::loadbalance::Metadata;

#[derive(Debug)]
pub struct ConsistentHash {
    metadata: Metadata,
}

impl ConsistentHash {
    pub fn new() -> ConsistentHash {
        ConsistentHash {
            metadata: Metadata {
                name: "consistenthash".to_string()
            }
        }
    }
}
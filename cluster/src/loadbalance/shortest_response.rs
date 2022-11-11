use crate::loadbalance::Metadata;

#[derive(Debug)]
pub struct ShortestResponse {
    metadata: Metadata,
}

impl ShortestResponse {
    pub fn new() -> ShortestResponse {
        ShortestResponse {
            metadata: Metadata {
                name: "shortestresponse".to_string()
            }
        }
    }
}
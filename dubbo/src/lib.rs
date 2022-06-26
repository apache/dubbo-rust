pub mod helloworld;
pub mod service;
pub mod common;
pub mod utils;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync>;
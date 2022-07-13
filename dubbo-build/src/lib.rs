

#[cfg(feature = "generator-code")]
extern crate prost_build;
#[cfg(feature = "generator-code")]
mod generator;
#[cfg(feature = "generator-code")]
pub use generator::CodeGenerator;

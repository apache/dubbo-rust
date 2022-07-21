use std::io::Result;
use prost_build::Config;

fn main() -> Result<()> {
    let mut conf = Config::new();
    let gen = dubbo_build::CodeGenerator::new();
    conf.service_generator(Box::new(gen));
    conf.out_dir("src/");
    conf.compile_protos(&["pb/greeter.proto"], &["pb/"]).unwrap();
    Ok(())
}
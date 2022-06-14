use std::io::Result;
use prost_build::Config;

fn main() -> Result<()> {

    Config::new()
        .out_dir("src/pb")
        .compile_protos(&["pb/person.proto"], &["pb/"])?;

    // println!("cargo:rerun-if-changed=pb/greeter_rpc.pb");
    // println!("cargo:rerun-if-changed=pb/build.rs");

    Config::new()
        .out_dir("src/pb")
        .compile_protos(&["pb/greeter.proto"], &["pb/"])?;

    Ok(())
}
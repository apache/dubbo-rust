# Apache Dubbo-rust 示例 - interface

## 构建并运行

```sh
$ cd github.com/apache/dubbo-rust/examples/interface/
$ cargo build

$ # run sever
$ ../../target/debug/interface-server

$ # run client
$ ../../target/debug/interface-client

# client stream
server response : Ok("Hello world1")
server response : Ok(ResDto { str: "Hello world2:world3 V2" })

# server stream
client request : "world1"
client request : ReqDto { str: "world2" } : ReqDto { str: "world3" }

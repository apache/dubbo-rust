# Apache Dubbo-rust 示例

## 构建并运行

```sh
$ cd github.com/apache/dubbo-rust/examples/echo/
$ cargo build

$ # 运行服务端
$ ../../target/debug/echo-server

$ # 运行客户端
$ ../../target/debug/echo-client
reply: EchoResponse { message: "msg1 from server" }
reply: EchoResponse { message: "msg2 from server" }
reply: EchoResponse { message: "msg3 from server" }
```

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

## 构建并运行`echo-tls`

**请先将`fixtures`路径下的`ca.crt`证书文件安装到系统信任根证书中.**

```sh
$ cd github.com/apache/dubbo-rust/examples/echo-tls/
$ cargo build

$ # 运行服务端
$ ../../target/debug/echo-tls-server

$ # 运行客户端
$ ../../target/debug/echo-tls-client
reply: EchoResponse { message: "msg1 from tls-server" }
reply: EchoResponse { message: "msg2 from tls-server" }
reply: EchoResponse { message: "msg3 from tls-server" }
```
# Apache Dubbo-rust example

## build and run

```sh
$ cd github.com/apache/dubbo-rust/examples/echo/
$ cargo build

$ # run sever
$ ../../target/debug/echo-server

$ # run client
$ ../../target/debug/echo-client
reply: EchoResponse { message: "msg1 from server" }
reply: EchoResponse { message: "msg2 from server" }
reply: EchoResponse { message: "msg3 from server" }
```

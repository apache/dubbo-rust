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

## build and run `echo-tls`

**Please first install the `ca.crt` certificate file under the `fixtures` path to the platform's native certificate store.**

```sh
$ cd github.com/apache/dubbo-rust/examples/echo-tls/
$ cargo build

$ # run sever
$ ../../target/debug/echo-tls-server

$ # run client
$ ../../target/debug/echo-tls-client
reply: EchoResponse { message: "msg1 from tls-server" }
reply: EchoResponse { message: "msg2 from tls-server" }
reply: EchoResponse { message: "msg3 from tls-server" }
```

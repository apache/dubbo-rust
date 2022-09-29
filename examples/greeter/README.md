# Apache Dubbo-rust example - greeter

## build and run

```sh
$ cd github.com/apache/dubbo-rust/examples/greeter/
$ cargo build

$ # run sever
$ ../../target/debug/greeter-server

$ # run client
$ ../../target/debug/greeter-client
# unary call
Response: GreeterReply { message: "hello, dubbo-rust" }
# client stream
client streaming, Response: GreeterReply { message: "hello client streaming" }
# bi stream
parts: Metadata { inner: {"date": "Wed, 28 Sep 2022 08:36:50 GMT", "content-type": "application/grpc"} }
reply: GreeterReply { message: "server reply: \"msg1 from client\"" }
reply: GreeterReply { message: "server reply: \"msg2 from client\"" }
reply: GreeterReply { message: "server reply: \"msg3 from client\"" }
trailer: Some(Metadata { inner: {"grpc-message": "poll trailer successfully.", "grpc-accept-encoding": "gzip,identity", "grpc-status": "0", "content-type": "application/grpc"} })
# server stream
parts: Metadata { inner: {"content-type": "application/grpc", "date": "Wed, 28 Sep 2022 08:36:50 GMT"} }
reply: GreeterReply { message: "msg1 from server" }
reply: GreeterReply { message: "msg2 from server" }
reply: GreeterReply { message: "msg3 from server" }
trailer: Some(Metadata { inner: {"content-type": "application/grpc", "grpc-status": "0", "grpc-accept-encoding": "gzip,identity", "grpc-message": "poll trailer successfully."} })
```

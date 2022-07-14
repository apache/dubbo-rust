dubbo-rust
=========

Rust implementation of [gRPC] protocol for [Dubbo], under development.


[Examples] | [Docs] 

## Overview

[`dubbo-rust`] is composed of three main components: the generic gRPC implementation, the gRPC codegen 
implementation with dubbo-build powered by [`prost`]. The transporter layer implementation can support any HTTP/2
implementation is based on [`hyper`], [`tower`]. The message layer implementation can support any user defined bytes format.

## Features
- async/await service trait define
- generate gRPC code for easy use
- message byte define 

## Getting Started
Examples can be found in [`examples`]


### Tutorials

- The [`protobuf-transport`] example provides a basic example for testing transport protobuf-message-object-data via HTTP/2.
- The [`grpc-gen`] example provides a complete example of using bubbo-rust client/server grpc.

### Contribution

[gRPC]: https://grpc.io
[dubbo]: https://dubbo.apache.org/en/
[`prost`]: https://github.com/tokio-rs/prost
[`hyper`]: https://github.com/hyperium/hyper
[`tower`]: https://github.com/tower-rs/tower
[Examples]: https://github.com/Johankoi/dubbo-rust/tree/main/examples
[`examples`]: https://github.com/Johankoi/dubbo-rust/tree/main/examples
[`protobuf-transport`]: https://github.com/Johankoi/dubbo-rust/tree/main/examples/protobuf-transport
[`grpc-gen`]: https://github.com/Johankoi/dubbo-rust/tree/main/examples/grpc-gen




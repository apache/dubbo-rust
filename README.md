# Apache Dubbo-rust

<a href="https://dubbo.apache.org/">
    <img style="vertical-align: top;" src="https://dubbo.apache.org/imgs/dubbo_colorful.png" alt="logo" height="45px"></a>

Apache Dubbo-rust, an RPC framework that implements Dubbo written in Rust.Please visit the [official website](https://dubbo.apache.org/) for more information.

[![Build Status](https://img.shields.io/github/actions/workflow/status/apache/dubbo-rust/.github/workflows/github-actions.yml?branch=main&style=flat-square)](https://github.com/apache/dubbo-rust/actions/workflows/github-actions.yml?query=branch%3Amain)
![License](https://img.shields.io/github/license/apache/dubbo-rust?style=flat-square)

[ [中文](./README_CN.md) ]

## Overview

Dubbo-rust is still under development. For now, gRPC calls based on HTTP2 have been implemented.

The following libraries are mainly dependent on:

- [`Tokio`](https://github.com/tokio-rs/tokio) is an event-driven, non-blocking I/O platform for writing asynchronous applications with Rust.

- [`Prost`](https://github.com/tokio-rs/prost/) is a [Protocol Buffers](https://developers.google.com/protocol-buffers/) implementation for Rust.

- [`Hyper`](https://github.com/hyperium/hyper) is a fast and correct HTTP implementation for Rust.

- [`Serde`](https://github.com/serde-rs/serde) is a framework for *ser*ializing and *de*serializing Rust data structures efficiently and generically.

## Features

- :white_check_mark: RPC synchronous / asynchronous call 
- :white_check_mark: IDL code automatic generation
- :construction: Multiple RPC protocol support (like Triple, Dubbo, gRPC, JSONRPC)
- :construction: Support TCP/HTTP2 transport protocol
- :construction: Service registration and discovery

## Get started

- Dubbo-rust Quick Start:  [中文](https://dubbo.apache.org/zh/docs3-v2/rust-sdk/quick-start/), English
- Dubbo-rust Tutorials:  [Examples](https://github.com/apache/dubbo-rust/tree/main/examples)

## Project structure

```
.
├── Cargo.toml
├── LICENSE
├── README.md
├── README_CN.md
├── common
│   ├── Cargo.toml
│   └── src
│       └── lib.rs
├── config
│   ├── Cargo.toml
│   └── src
│       ├── config.rs
│       ├── lib.rs
│       ├── protocol.rs
│       └── service.rs
├── contributing.md
├── docs
│   ├── filter-design.md
│   ├── generic-protocol-design.md
│   ├── readme.md
│   └── services.md
more ...
```

## Contact Us

- Subscribe to the official Wechat Account
![officialAccount](https://user-images.githubusercontent.com/18097545/201456442-68a7bf1e-3c84-4f32-bd45-0fedb4d1012d.png)

- Search and join the DingTalk group: 44694199

## Contribute

Welcome more developers to join us. About more details please check "[How to contribute](https://github.com/apache/dubbo-rust/blob/main/CONTRIBUTING.md)".

## License

Apache Dubbo-rust software is licenced under the Apache License Version 2.0. See the [LICENSE](https://github.com/apache/dubbo-rust/blob/main/LICENSE) file for details.

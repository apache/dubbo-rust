# Apache Dubbo-rust

<a href="https://dubbo.apache.org/">
    <img style="vertical-align: top;" src="https://dubbo.apache.org/imgs/dubbo_colorful.png" alt="logo" height="45px"></a>

Apache Dubbo-rust, Dubbo RPC框架的Rust实现。请访问 [Dubbo官网](https://dubbo.apache.org/) 查看更多信息.

[![Build Status](https://travis-ci.org/apache/dubbo-rust.svg?branch=main)](https://travis-ci.org/apache/dubbo-rust) ![License](https://img.shields.io/github/license/alibaba/dubbo.svg)

## 概述

Dubbo-rust 目前还在开发阶段. 截至目前, 已经实现了基于HTTP2的gRPC调用.

以下为主要的依赖库:

- [`Tokio`](https://github.com/tokio-rs/tokio) 使用Rust编写事件驱动、无阻塞I/O异步程序的框架。

- [`Prost`](https://github.com/tokio-rs/prost/)  [Protocol Buffers](https://developers.google.com/protocol-buffers/) Rust实现。

- [`Hyper`](https://github.com/hyperium/hyperhttps://github.com/hyperium/hyper) 构建HTTP协议的Rust库。

- [`Serde`](https://github.com/serde-rs/serde) 序列化/反序列化Rust库

## 功能列表

- :white_check_mark: RPC 异步/同步调用
- :white_check_mark: IDL文件代码生成器
- :construction: RPC多协议支持（如： Triple, Dubbo, gRPC, JSONRPC）
- :construction: 支持 TCP/HTTP2 传输层协议
- :construction: 服务注册与发现

## 开始使用

- Dubbo-rust 快速开始:  [中文](https://dubbo.apache.org/zh/docs3-v2/rust-sdk/quick-start/), English
- Dubbo-rust 教程:  [Examples](https://github.com/apache/dubbo-rust/tree/main/examples)

## 项目结构

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

## 联系方式

- [钉钉](https://www.dingtalk.com/enhttps://www.dingtalk.com/en):  44694199

## 贡献

欢迎更多的开发者加入我们。关于更多的信息可以查看 [[CONTRIBUTING](https://github.com/apache/dubbo-rust/blob/main/contributing.md)]。

## 许可证

Apache Dubbo-rust 使用Apache许可证2.0版本。 请参考 [LICENSE](https://github.com/apache/dubbo-rust/blob/main/LICENSE) 文件获得更多信息。
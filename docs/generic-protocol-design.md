author: Yang Yang
date: 2022-06-26

## 简介

dubbo-rust支持多种协议：Triple、gRPC、jsonRPC等

## Protocol设计

Protocol的核心设计是基于dubbo的URL模型，对外暴露通用的服务端和客户端抽象接口。

在Dubbo的整体生态中，服务端接口使用`Exporter`来描述；客户端接口使用`Invoker`来描述。

Protocol模块的核心功能：
+ 对外提供服务注册接口
+ 管理注册的服务：run, destroy, stop, gracefulStop
+ 接口路由
+ 通用、高效的Listener层
+ 等等

### Exporter

### Invoker

Invoker提供的通用的接口，使得dubbo在不同的协议下遵循相同的接口抽象。

在Invoker中，需要做的功能包括
+ 编解码层
+ Streaming trait实现
+ 自定义请求/响应
+ 等等

## 目前存在的问题

+ 如何管理服务：服务是动态的，需要保证Server是Send+Sync的
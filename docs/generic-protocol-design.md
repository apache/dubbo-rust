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

Protocol API支持管理多个底层协议的server以及在一个server上暴露多个服务
+ 多个底层通信的server：location(ip:port): server
+ 一个server上暴露多个服务：

### Exporter

### Invoker

Invoker: 客户端通用能力的封装。获取需要一个private withInvoker 接口

Invoker应该是基于Connection(Service)实现的Service。并且进行扩展新的接口。
protocol.rs模块：根据Url返回初始化好的Invoker实例。
+ 如何设计Invoker接口：扩展Service接口
+ cluster模块如何使用Invoker实例呢？这里需要画一个数据流转图
+ 如何将初始化好的Invoker与tower::Layer相结合

Invoker提供的通用的接口，使得dubbo在不同的协议下遵循相同的接口抽象。

在Invoker中，需要做的功能包括
+ 编解码层
+ Streaming trait实现
+ 自定义请求/响应
+ 等等

## 目前存在的问题

+ 如何管理服务：服务是动态的，需要保证Server是Send+Sync的
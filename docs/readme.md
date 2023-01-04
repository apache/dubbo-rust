# Readme

There is some RFCs of dubbo-rust design.

## 关于配置的一些约定(暂时)

所有的服务只能注册到一个或多个注册中心
所有的服务只能使用Triple进行通信
Triple只能对外暴露一个端口

一个协议上的server注册到
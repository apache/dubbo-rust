## 关于配置的一些约定(暂时)

所有的服务只能注册到一个或多个注册中心
所有的服务只能使用Triple进行通信
Triple只能对外暴露一个端口

## Config配置

每个组件的配置是独立的。

Provider、Consumer等使用独立组件的配置进行工作

Provider Config核心设计以及Url模型流转：

Provider和Consumer使用组件的配置
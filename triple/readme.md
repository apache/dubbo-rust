# Triple Protocol

Triple协议使用了hyper作为C/S之间的通信层，支持triple spec中自定义的header。

整体模块划分为：
+ codec
+ server:
  + Router
  + 网络事件处理层
  + 线程池：参考triple-go的线程模型
+ client:
  + 线程池
  + 连接处理层，类似于grpc channel层

Triple支持tls、grpc压缩
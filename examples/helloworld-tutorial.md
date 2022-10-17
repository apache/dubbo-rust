# Dubbo-rust quick start

本文介绍如何使用Rust快速开发Dubbo服务。

主要内容分为：
+ 环境准备
+ 使用Protobuf IDL定义Dubbo服务
+ 使用dubbo-build来编译IDL
+ 编写Dubbo业务代码

## 环境准备

首先需要安装Rust开发环境，具体步骤为：
```
```

其次还需要下载protoc可执行文件，并将protoc添加到Path环境变量中：
下载地址为：`https://github.com/protocolbuffers/protobuf/releases`

```
export Path = ""
```

## 使用Protobuf IDL定义Dubbo服务

## 使用dubbo-build来编译IDL

首先创建一个项目：
```
cargo new --lib dubbo-example
```

在`cargo.toml`中添加需要的依赖
```
[dependencies]
http = "0.2"
http-body = "0.4.4"
futures-util = {version = "0.3", default-features = false}
tokio = { version = "1.0", features = [ "rt-multi-thread", "time", "fs", "macros", "net", "signal"] }
prost-derive = {version = "0.10", optional = true}
prost = "0.10.4"
prost-types = { version = "0.8", default-features = false }
async-trait = "0.1.56"
tokio-stream = "0.1"

dubbo = {git = "https://github.com/yang20150702/dubbo-rust.git"}

[build-dependencies]
dubbo-build = {git = "https://github.com/yang20150702/dubbo-rust.git"}
```

将写好的`helloworld.proto`放在proto/目录下。

编写`build.rs`文件来生成`helloworld.proto`对应的rust代码：
```
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from("./src");
    dubbo_build::prost::configure()
        .output_dir(path)
        .compile(&["proto/helloworld.proto"], &["proto/"])
        .unwrap();
}
```

此时在src目录下会有一个名为`helloworld.rs`的文件，这就是生成的rust代码。

> `helloworld.rs`可以放在src目录下的任一位置，只要它能够被业务代码引用到即可~

## 编写Dubbo业务代码

Dubbo业务代码分为server和client两部分。

首先在lib.rs目录下引入`helloworld.rs`mod：
```
pub mod helloworld;
```

然后，编写`dubbo.yaml`配置文件，将该文件放在src目录的同级目录中：
```
name: dubbo
service:
  helloworld.Greeter:
    version: 1.0.0
    group: test
    protocol: triple
    registry: ''
    serializer: json
    protocol_configs: {}
protocols:
  triple:
    ip: 0.0.0.0
    port: '8888'
    name: triple
```

> 另外，可以通过环境变量`DUBBO_CONFIG_PATH`来自定义配置文件的路径。

### 编写 Dubbo Server

在src目录下新建server文件夹，创建`main.rs`文件，并在`Cargo.toml`中增加如下配置：
```
[[bin]]
name = "hello-server"
path = "src/server/main.rs"
```

首先引入需要的模块：
```
use dubbo_rust_examples::helloworld::greeter_server::{register_server, Greeter};
use dubbo_rust_examples::helloworld::{HelloRequest, HelloReply};
use async_trait::async_trait;
use dubbo::codegen::{Request, Response};
use dubbo::Dubbo;
```

接下来，实现IDL中定义的服务trait：
```
#[derive(Debug, Clone, Default)]
pub struct GreeterImpl {}

impl GreeterImpl {
    pub fn new() -> Self {
        GreeterImpl {  }
    }
}

#[async_trait]
impl Greeter for GreeterImpl {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, dubbo::status::Status> {
        println!("request: {:?}", request.into_inner());

        Ok(Response::new(HelloReply{
            message: "hello dubbo-rust!".to_string(),
        }))
    }
}
```

接下来进行服务注册，并启动Dubbo框架：
```
#[tokio::main]
async fn main() {
    register_server(GreeterImpl::new());

    Dubbo::new().start().await;
}
```

通过如下命令启动服务：
```
cargo run --bin hello-server
```

### 编写 Dubbo Client

在src目录下新建client文件夹，创建`main.rs`文件，并在`Cargo.toml`中增加如下配置：
```
[[bin]]
name = "hello-client"
path = "src/client/main.rs"
```

首先引入需要的模块：
```
use dubbo_rust_examples::helloworld::greeter_client::GreeterClient;
use dubbo_rust_examples::helloworld::HelloRequest;
use dubbo::codegen::Request;
```

接下来，实现IDL中定义的服务trait：
```
#[tokio::main]
async fn main() {
    let mut cli = GreeterClient::new().with_uri("http://127.0.0.1:8888".to_string());
    let resp = cli.say_hello(Request::new(
        HelloRequest{
            name: "hello, I'm client".to_string(),
        }
    )).await.unwrap();

    let (_, msg) = resp.into_parts();
    println!("response: {:?}", msg);
}
```

通过如下命令启动服务：
```
cargo run --bin hello-client
```
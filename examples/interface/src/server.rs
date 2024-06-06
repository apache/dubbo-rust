/*
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.
 * The ASF licenses this file to You under the Apache License, Version 2.0
 * (the "License"); you may not use this file except in compliance with
 * the License.  You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::env;

use dubbo::{
    config::RootConfig,
    extension,
    logger::{tracing::span, Level},
    Dubbo,
};
use dubbo_macro::dubbo_server;
use example_interface::{DemoService, ReqDto, ResDto};
use registry_nacos::NacosRegistry;

#[derive(Clone)]
struct DemoServiceImpl {
    _db: String,
}

#[dubbo_server(package = "org.apache.dubbo.springboot.demo")]
impl DemoService for DemoServiceImpl {
    async fn sayHello(&self, req: String) -> Result<String, dubbo::status::Status> {
        println!("client request : {:?}", req);
        Ok("Hello ".to_owned() + &req)
    }
    async fn sayHelloV2(&self, req: ReqDto, req2: ReqDto) -> Result<ResDto, dubbo::status::Status> {
        println!("client request : {:?} : {:?}", req, req2);
        Ok(ResDto {
            str: "Hello ".to_owned() + &req.str + ":" + &req2.str + " V2",
        })
    }
}

#[tokio::main]
async fn main() {
    dubbo::logger::init();
    let span = span!(Level::DEBUG, "interface.server");
    env::set_var("DUBBO_CONFIG_PATH", "examples/interface/application.yaml");
    let _enter = span.enter();
    let r = RootConfig::new();
    let r = match r.load() {
        Ok(config) => config,
        Err(_err) => panic!("err: {:?}", _err), // response was droped
    };
    let _ = extension::EXTENSIONS.register::<NacosRegistry>().await;
    let server = DemoServiceImpl {
        _db: "i am db".to_owned(),
    };
    let mut f = Dubbo::new()
        .with_config(r)
        .add_registry("nacos://127.0.0.1:8848/")
        .register_server(server);
    f.start().await;
}

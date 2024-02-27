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

use dubbo::codegen::{ClientBuilder, RegistryDirectory};
use example_interface::{DemoServiceRpc, ReqDto};
use registry_zookeeper::ZookeeperRegistry;

#[tokio::main]
async fn main() {
    // dubbo_logger::init();
    let zkr = ZookeeperRegistry::new("127.0.0.1:2181");
    let directory = RegistryDirectory::new(Box::new(zkr));
    let builder = ClientBuilder::new().with_directory(Box::new(directory));
    let mut client = DemoServiceRpc::new(builder);
    let res = client.sayHello("world1".to_string()).await;
    println!("server response : {:?}", res);
    let res = client
        .sayHelloV2(
            ReqDto {
                str: "world2".to_string(),
            },
            ReqDto {
                str: "world3".to_string(),
            },
        )
        .await;
    println!("server response : {:?}", res);
}

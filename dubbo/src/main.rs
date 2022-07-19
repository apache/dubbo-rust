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

pub mod common;
pub mod echo;
pub mod helloworld;
pub mod init;
pub mod protocol;
pub mod registry;
pub mod utils;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() {
    println!("hello, dubbo-rust~")
}

// Refer service: tri://localhost:20000/helloworld.Greeter?app.version=3.0.0&application=dubbo.io&async=false&bean.name=GreeterClientImpl&cluster=failover&config.tracing=&environment=dev&generic=&group=&interface=helloworld.Greeter&loadbalance=&metadata-type=local&module=sample&name=dubbo.io&organization=dubbo-go&owner=dubbo-go&provided-by=&reference.filter=cshutdown&registry.role=0&release=dubbo-golang-3.0.0&retries=&serialization=&side=consumer&sticky=false&timestamp=1657865138&version=

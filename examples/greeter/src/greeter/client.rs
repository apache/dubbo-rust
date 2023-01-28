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

use std::{env, str::FromStr, time::Duration};

use http;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use dubbo::cluster::support::cluster_invoker::ClusterInvoker;
use dubbo::{cluster::directory::RegistryDirectory, codegen::*};
use dubbo_registry_zookeeper::zookeeper_registry::ZookeeperRegistry;
use protos::{greeter_client::GreeterClient, GreeterRequest};

pub mod protos {
    #![allow(non_camel_case_types)]
    include!(concat!(env!("OUT_DIR"), "/org.apache.dubbo.sample.tri.rs"));
}

#[tokio::main]
async fn main() {
    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::DEBUG)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let zk_connect_string = match env::var("ZOOKEEPER_SERVERS") {
        Ok(val) => val,
        Err(_) => {
            let default_connect_string = "localhost:2181";
            info!(
                "No ZOOKEEPER_SERVERS env value, using {} as default.",
                default_connect_string
            );
            default_connect_string.to_string()
        }
    };
    let zkr = ZookeeperRegistry::new(&zk_connect_string);
    let directory = RegistryDirectory::new(Box::new(zkr));
    let cluster_invoker = ClusterInvoker::with_directory(directory);
    let http_uri = http::Uri::from_str(&"http://1.1.1.1:8888").unwrap();

    let mut cli = GreeterClient::new(Connection::new().with_host(http_uri));
    //cli = cli.with_directory(Box::new(directory));
    cli = cli.with_cluster(cluster_invoker);
    // let mut cli = GreeterClient::connect("http://127.0.0.1:8888".to_string());
    // using loop for loadbalance test
    for _ in 0..10 {
        println!("# unary call");
        let resp = cli
            .greet(Request::new(GreeterRequest {
                name: "message from client".to_string(),
            }))
            .await;
        let resp = match resp {
            Ok(resp) => resp,
            Err(err) => return println!("{:?}", err),
        };
        let (_parts, body) = resp.into_parts();
        println!("Response: {:?}", body);

        tokio::time::sleep(Duration::from_millis(2000)).await;
    }
}

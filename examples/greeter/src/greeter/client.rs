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

pub mod protos {
    #![allow(non_camel_case_types)]
    include!(concat!(env!("OUT_DIR"), "/org.apache.dubbo.sample.tri.rs"));
}

use dubbo::codegen::*;
use futures_util::StreamExt;
use protos::{greeter_client::GreeterClient, GreeterRequest};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::INFO)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let mut cli = GreeterClient::new(ClientBuilder::from_static(&"http://127.0.0.1:8888"));

    // Here is example for zk
    // let zk_connect_string = match env::var("ZOOKEEPER_SERVERS") {
    //     Ok(val) => val,
    //     Err(_) => "localhost:2181".to_string(),
    // };
    // let zkr = ZookeeperRegistry::new(&zk_connect_string);
    // let directory = RegistryDirectory::new(Box::new(zkr));
    // cli = cli.with_directory(Box::new(directory));

    let zkr = ZookeeperRegistry::default();
    let directory = RegistryDirectory::new(Box::new(zkr));
    let cluster_invoker = ClusterInvoker::with_directory(directory);

    let mut cli = GreeterClient::new(Connection::new());
    cli = cli.with_cluster(cluster_invoker);
    // using loop for loadbalance test
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

    println!("# client stream");
    let data = vec![
        GreeterRequest {
            name: "msg1 from client streaming".to_string(),
        },
        GreeterRequest {
            name: "msg2 from client streaming".to_string(),
        },
        GreeterRequest {
            name: "msg3 from client streaming".to_string(),
        },
    ];
    let req = futures_util::stream::iter(data);
    let resp = cli.greet_client_stream(req).await;
    let client_streaming_resp = match resp {
        Ok(resp) => resp,
        Err(err) => return println!("{:?}", err),
    };
    let (_parts, resp_body) = client_streaming_resp.into_parts();
    println!("client streaming, Response: {:?}", resp_body);

    println!("# bi stream");
    let data = vec![
        GreeterRequest {
            name: "msg1 from client".to_string(),
        },
        GreeterRequest {
            name: "msg2 from client".to_string(),
        },
        GreeterRequest {
            name: "msg3 from client".to_string(),
        },
    ];
    let req = futures_util::stream::iter(data);

    let bidi_resp = cli.greet_stream(req).await.unwrap();

    let (parts, mut body) = bidi_resp.into_parts();
    println!("parts: {:?}", parts);
    while let Some(item) = body.next().await {
        match item {
            Ok(v) => {
                println!("reply: {:?}", v);
            }
            Err(err) => {
                println!("err: {:?}", err);
            }
        }
    }
    let trailer = body.trailer().await.unwrap();
    println!("trailer: {:?}", trailer);

    println!("# server stream");
    let resp = cli
        .greet_server_stream(Request::new(GreeterRequest {
            name: "server streaming req".to_string(),
        }))
        .await
        .unwrap();

    let (parts, mut body) = resp.into_parts();
    println!("parts: {:?}", parts);
    while let Some(item) = body.next().await {
        match item {
            Ok(v) => {
                println!("reply: {:?}", v);
            }
            Err(err) => {
                println!("err: {:?}", err);
            }
        }
    }
    let trailer = body.trailer().await.unwrap();
    println!("trailer: {:?}", trailer);
}

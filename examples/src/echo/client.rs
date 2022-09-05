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

use dubbo::codegen::*;
use examples::protos::hello_echo::echo_client::EchoClient;
use examples::protos::hello_echo::EchoRequest;
use futures_util::StreamExt;

#[tokio::main]
async fn main() {
    let mut cli = EchoClient::new().with_uri("http://127.0.0.1:8888".to_string());
    let resp = cli
        .unary_echo(Request::new(EchoRequest {
            message: "message from client".to_string(),
        }))
        .await;
    let resp = match resp {
        Ok(resp) => resp,
        Err(err) => return println!("{:?}", err),
    };
    let (_parts, body) = resp.into_parts();
    println!("Response: {:?}", body);

    let data = vec![
        EchoRequest {
            message: "msg1 from client streaming".to_string(),
        },
        EchoRequest {
            message: "msg2 from client streaming".to_string(),
        },
        EchoRequest {
            message: "msg3 from client streaming".to_string(),
        },
    ];
    let req = futures_util::stream::iter(data);
    let resp = cli.client_streaming_echo(req).await;
    let client_streaming_resp = match resp {
        Ok(resp) => resp,
        Err(err) => return println!("{:?}", err),
    };
    let (_parts, resp_body) = client_streaming_resp.into_parts();
    println!("client streaming, Response: {:?}", resp_body);

    let data = vec![
        EchoRequest {
            message: "msg1 from client".to_string(),
        },
        EchoRequest {
            message: "msg2 from client".to_string(),
        },
        EchoRequest {
            message: "msg3 from client".to_string(),
        },
    ];
    let req = futures_util::stream::iter(data);

    let bidi_resp = cli.bidirectional_streaming_echo(req).await.unwrap();

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

    let resp = cli
        .server_streaming_echo(Request::new(EchoRequest {
            message: "server streaming req".to_string(),
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

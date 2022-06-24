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

mod addservice;
mod mutilservice;

use std::{net::SocketAddr, str::FromStr, time::Duration};

use addservice::{add_client::AddClient, AddReq};
use mutilservice::{mutil_client::MutilClient, MutilReq};

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from_str("127.0.0.1:40022").unwrap();

    let mut client_add = AddClient::new(&addr).unwrap();
    let mut client_mutil = MutilClient::new(&addr).unwrap();
    loop {
        let add_req = AddReq {
            numbers: vec![1, 2, 21],
        };
        let resp = client_add.add(add_req).await.unwrap();

        let mutil_req = MutilReq {
            numbers: vec![1, 20, 30],
        };
        let resp2 = client_mutil.mutil(mutil_req).await.unwrap();

        println!("add resp : {:?}", resp);
        println!("mutil resp : {:?}", resp2);

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

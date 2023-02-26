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

use crate::dial::DefaultMakeTransport;
use crate::Address;
use tokio::io::AsyncWriteExt;

// listen by command: `nc -l 8858 -v`
#[tokio::test(flavor = "current_thread")]
async fn test_tcp_bytes_send() {
    let transport = DefaultMakeTransport::new();
    let mut conn = transport
        .make_connection(Address::Ip("127.0.0.1:8858".parse().unwrap()))
        .await
        .unwrap();
    conn.write_all("\n\rhello dubbo-rust\n\r".to_string().as_bytes())
        .await
        .unwrap();
}

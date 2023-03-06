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

use crate::error::CodecError;
use std::{any::Any, sync::Arc};
pub mod client;
pub mod server;

pub type BoxedExchangeBody = Arc<dyn Any>;

pub struct Request {
    id: u64,
    version: String, // protocol version
    serial_id: u8,   // serial ID (ignore)
    body: Option<BoxedExchangeBody>,
    two_way: bool,
    event: bool,
}

pub struct Response {
    id: u64,
    version: String, // protocol version
    serial_id: u8,   // serial ID (ignore)
    status: u8,
    body: Option<BoxedExchangeBody>, // mean result
    event: bool,
    error: Option<CodecError>,
}

impl Response {
    fn is_heart_beat(&self) -> bool {
        self.event && self.body.is_none()
    }
}

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

pub mod cluster;
pub mod codegen;
pub mod context;
pub mod filter;
mod framework;
pub mod invocation;
pub mod protocol;
pub mod registry;
pub mod status;
pub mod triple;
pub mod utils;

use http_body::Body;
use std::{future::Future, pin::Pin};

pub use framework::Dubbo;

pub type StdError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type BoxFuture<T, E> = self::Pin<Box<dyn self::Future<Output = Result<T, E>> + Send + 'static>>;
pub(crate) type Error = Box<dyn std::error::Error + Send + Sync>;
pub type BoxBody = http_body::combinators::UnsyncBoxBody<bytes::Bytes, self::status::Status>;

pub fn empty_body() -> BoxBody {
    http_body::Empty::new()
        .map_err(|err| match err {})
        .boxed_unsync()
}

pub(crate) fn boxed<B>(body: B) -> BoxBody
where
    B: http_body::Body<Data = bytes::Bytes> + Send + 'static,
    B::Error: Into<self::Error>,
{
    body.map_err(|err| {
        self::status::Status::new(self::status::Code::Internal, format!("{:?}", err.into()))
    })
    .boxed_unsync()
}

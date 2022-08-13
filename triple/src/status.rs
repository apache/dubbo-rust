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

use std::error::Error;
use std::fmt;

pub const GRPC_STATUS: &str = "grpc-status";
pub const GRPC_MESSAGE: &str = "grpc-message";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Code {
    Ok,
    Internal,
    Unimplemented,
    Unknown,
}

impl Code {
    pub fn description(&self) -> &'static str {
        match self {
            Code::Ok => "The operation completed successfully",
            Code::Unknown => "Unknown error",
            Code::Unimplemented => "Operation is not implemented or not supported",
            Code::Internal => "Internal error",
        }
    }
}

impl std::fmt::Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.description(), f)
    }
}

#[derive(Debug, Clone)]
pub struct Status {
    // grpc-status
    code: Code,

    // grpc-message
    message: String,
}

impl Status {
    pub fn new(code: Code, message: String) -> Self {
        Status { code, message }
    }

    pub fn with_message(self, message: String) -> Self {
        Status { message, ..self }
    }

    pub fn from_std_erro<T: std::error::Error>(err: T) -> Self {
        Status::new(Code::Internal, err.to_string())
    }

    pub fn from_error(err: crate::Error) -> Self {
        Status::new(Code::Internal, err.to_string())
    }

    pub fn code(&self) -> Code {
        self.code
    }

    pub fn to_http(&self) -> http::Response<crate::BoxBody> {
        let (mut parts, _) = http::Response::new(()).into_parts();

        parts.headers.insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("application/grpc"),
        );

        parts
            .headers
            .insert(GRPC_STATUS, http::HeaderValue::from_static("a"));
        parts.headers.insert(
            GRPC_MESSAGE,
            http::HeaderValue::from_str(&self.message).unwrap(),
        );

        parts.headers.insert(
            "grpc-accept-encoding",
            http::HeaderValue::from_static("gzip,identity"),
        );

        http::Response::from_parts(parts, crate::empty_body())
    }
}

impl From<std::io::Error> for Status {
    fn from(err: std::io::Error) -> Self {
        Status::new(crate::status::Code::Internal, err.to_string())
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "grpc status, code: {}, message: {}",
            self.code, self.message
        ))
    }
}

impl Error for Status {}

#[derive(Debug)]
pub struct DubboError {
    message: String,
}

impl DubboError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl fmt::Display for DubboError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Dubbo internal Error: {}", self.message)
    }
}

impl Error for DubboError {}

unsafe impl Send for DubboError {}

unsafe impl Sync for DubboError {}

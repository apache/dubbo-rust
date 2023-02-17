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

use std::{collections::HashMap, fmt::Debug, str::FromStr};

use futures_core::Stream;

pub struct Request<T> {
    pub message: T,
    pub metadata: Metadata,
}

impl<T> Request<T> {
    pub fn new(message: T) -> Request<T> {
        Self {
            message,
            metadata: Metadata::new(),
        }
    }

    pub fn into_inner(self) -> T {
        self.message
    }

    pub fn into_parts(self) -> (Metadata, T) {
        (self.metadata, self.message)
    }

    pub fn from_parts(metadata: Metadata, message: T) -> Self {
        Request { message, metadata }
    }

    pub fn from_http(req: http::Request<T>) -> Self {
        let (parts, body) = req.into_parts();
        Request {
            metadata: Metadata::from_headers(parts.headers),
            message: body,
        }
    }

    pub fn into_http(
        self,
        uri: http::Uri,
        method: http::Method,
        version: http::Version,
    ) -> http::Request<T> {
        let mut http_req = http::Request::new(self.message);
        *http_req.version_mut() = version;
        *http_req.uri_mut() = uri;
        *http_req.method_mut() = method;
        *http_req.headers_mut() = self.metadata.into_headers();

        http_req
    }

    pub fn map<F, U>(self, f: F) -> Request<U>
    where
        F: FnOnce(T) -> U,
    {
        let m = f(self.message);
        Request {
            message: m,
            metadata: self.metadata,
        }
    }
}

pub struct Response<T> {
    message: T,
    metadata: Metadata,
}

impl<T> Response<T> {
    pub fn new(message: T) -> Response<T> {
        Self {
            message,
            metadata: Metadata::new(),
        }
    }

    pub fn from_parts(metadata: Metadata, message: T) -> Self {
        Self { message, metadata }
    }

    pub fn into_parts(self) -> (Metadata, T) {
        (self.metadata, self.message)
    }

    pub fn into_http(self) -> http::Response<T> {
        let mut http_resp = http::Response::new(self.message);
        *http_resp.version_mut() = http::Version::HTTP_2;
        *http_resp.headers_mut() = self.metadata.into_headers();

        http_resp
    }

    pub fn from_http(resp: http::Response<T>) -> Self {
        let (part, body) = resp.into_parts();
        Response {
            message: body,
            metadata: Metadata::from_headers(part.headers),
        }
    }

    pub fn map<F, U>(self, f: F) -> Response<U>
    where
        F: FnOnce(T) -> U,
    {
        let u = f(self.message);
        Response {
            message: u,
            metadata: self.metadata,
        }
    }
}

pub trait IntoStreamingRequest {
    type Stream: Stream<Item = Self::Message> + Send + 'static;
    type Message;

    fn into_streaming_request(self) -> Request<Self::Stream>;
}

impl<T> IntoStreamingRequest for T
where
    T: Stream + Send + 'static,
    // T::Item: Result<Self::Message, std::convert::Infallible>,
{
    type Stream = T;

    type Message = T::Item;

    fn into_streaming_request(self) -> Request<Self::Stream> {
        Request::new(self)
    }
}

// impl<T> sealed::Sealed for T {}

// pub mod sealed {
//     pub trait Sealed {}
// }

#[derive(Debug, Clone, Default)]
pub struct Metadata {
    inner: HashMap<String, String>,
}

impl Metadata {
    pub fn new() -> Self {
        Metadata {
            inner: HashMap::new(),
        }
    }

    pub fn from_headers(headers: http::HeaderMap) -> Self {
        let mut h: HashMap<String, String> = HashMap::new();
        for (k, v) in headers.into_iter() {
            if let Some(name) = k {
                h.insert(name.to_string(), v.to_str().unwrap().to_string());
            }
        }

        Metadata { inner: h }
    }

    pub fn into_headers(&self) -> http::HeaderMap {
        let mut header = http::HeaderMap::new();
        for (k, v) in self.inner.clone().into_iter() {
            header.insert(
                http::header::HeaderName::from_str(k.as_str()).unwrap(),
                http::HeaderValue::from_str(v.as_str()).unwrap(),
            );
        }

        header
    }
}

pub trait Invocation {
    fn get_target_service_unique_name(&self) -> String;
    fn get_method_name(&self) -> String;
}

#[derive(Default)]
pub struct RpcInvocation {
    target_service_unique_name: String,
    method_name: String,
}

impl RpcInvocation {
    pub fn with_service_unique_name(mut self, service_unique_name: String) -> Self {
        self.target_service_unique_name = service_unique_name;
        self
    }
    pub fn with_method_name(mut self, method_name: String) -> Self {
        self.method_name = method_name;
        self
    }
    pub fn unique_fingerprint(&self) -> String {
        format!("{}#{}", self.target_service_unique_name, self.method_name)
    }
}

impl Invocation for RpcInvocation {
    fn get_target_service_unique_name(&self) -> String {
        self.target_service_unique_name.clone()
    }

    fn get_method_name(&self) -> String {
        self.method_name.clone()
    }
}

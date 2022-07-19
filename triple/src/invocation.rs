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

use futures_core::Stream;
use tonic::metadata::MetadataMap;

pub struct Request<T> {
    pub message: T,
    pub metadata: MetadataMap,
}

impl<T> Request<T> {
    pub fn new(message: T) -> Request<T> {
        Self {
            message,
            metadata: MetadataMap::new(),
        }
    }

    pub fn into_inner(self) -> T {
        self.message
    }

    pub fn into_parts(self) -> (MetadataMap, T) {
        (self.metadata, self.message)
    }

    pub fn from_http(req: http::Request<T>) -> Self {
        let (parts, body) = req.into_parts();
        Request {
            metadata: MetadataMap::from_headers(parts.headers),
            message: body,
        }
    }

    pub fn into_http(self) -> http::Request<T> {
        let mut http_req = http::Request::new(self.message);
        *http_req.version_mut() = http::Version::HTTP_2;
        *http_req.headers_mut() = self.metadata.into_headers();

        http_req
    }
}

pub struct Response<T> {
    message: T,
    metadata: MetadataMap,
}

impl<T> Response<T> {
    pub fn new(message: T) -> Response<T> {
        Self {
            message,
            metadata: MetadataMap::new(),
        }
    }

    pub fn from_parts(metadata: MetadataMap, message: T) -> Self {
        Self { message, metadata }
    }

    pub fn into_parts(self) -> (MetadataMap, T) {
        (self.metadata, self.message)
    }

    pub fn into_http(self) -> http::Response<T> {
        let mut http_resp = http::Response::new(self.message);
        *http_resp.version_mut() = http::Version::HTTP_2;
        *http_resp.headers_mut() = self.metadata.into_headers();

        http_resp
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

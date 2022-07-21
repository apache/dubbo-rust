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

use std::convert::From;
use hyper::{body, Body, Request, Uri, Method, Version, header};
use prost::{Message};
use crate::error::*;

/// A request with HTTP info and the serialized input object
#[derive(Debug)]
pub struct ServiceRequest<T> {
    /// The URI of the original request
    ///
    /// When using a client, this will be overridden with the proper URI. It is only valuable for servers.
    pub uri: Uri,
    /// The request method; should always be Post
    pub method: Method,
    /// The HTTP version, rarely changed from the default
    pub version: Version,
    /// The set of headers
    ///
    /// Should always at least have `Content-Type`. Clients will override `Content-Length` on serialization.
    pub headers: header::HeaderMap,
    // The serialized request object
    pub input: T,
}

impl<T: Message + Default + 'static> ServiceRequest<T> {
    /// Create new service request with the given input object
    /// This automatically sets the `Content-Type` header as `application/protobuf`.
    pub fn new(input: T, uri: Uri) -> ServiceRequest<T> {
        let mut header = header::HeaderMap::new();
        header.insert(header::CONTENT_TYPE, "application/protobuf".parse().unwrap());
        ServiceRequest {
            uri,
            method: Method::POST,
            version: Version::default(),
            headers: header,
            input,
        }
    }

    /// Turn a protobuf service request into a byte-array service request
    pub fn try_encode(&self) -> Result<ServiceRequest<Vec<u8>>, DBProstError> {
        let mut body = Vec::new();
        self.input.encode(&mut body)
            .map(|v|
                ServiceRequest {
                    uri: self.uri.clone(),
                    method: self.method.clone(),
                    version: self.version,
                    headers: self.headers.clone(),
                    input: body,
                })
            .map_err(|e| DBProstError::ProstEncodeError(e))
    }

    /// Turn a protobuf service request into a hyper request
    pub fn into_encoded_hyper(&self) -> Result<Request<Body>, DBProstError> {
        self.try_encode().map(|v| v.into_hyper())
    }
}

impl ServiceRequest<Vec<u8>> {
    /// Turn a hyper request to ServiceRequest<Vec<u8>>
    pub async fn try_from_hyper(req: Request<Body>) -> Result<ServiceRequest<Vec<u8>>, DBProstError> {
        let uri = req.uri().clone();
        let method = req.method().clone();
        let version = req.version();
        let headers = req.headers().clone();
        body::to_bytes(req.into_body()).await
            .map_err(DBProstError::HyperError)
            .map(move |body| ServiceRequest { uri, method, version, headers, input: body.to_vec() })
    }

    /// Turn ServiceRequest<Vec<u8>> into a hyper request
    pub fn into_hyper(&self) -> Request<Body> {
        let mut request = Request::builder()
            .method(self.method.clone())
            .uri(self.uri.clone())
            .body(Body::from(self.input.clone())).unwrap();

        let header_mut = request.headers_mut();
        header_mut.clone_from(&self.headers);
        let lenth_value = header::HeaderValue::from(self.input.len() as u64);
        header_mut.insert(header::CONTENT_LENGTH, lenth_value);
        request
    }

    /// try_decode ServiceRequest<Vec<u8>> whose input is protobuf data into a ServiceRequest<T>
    pub fn try_decode<T: Message + Default + 'static>(&self) -> Result<ServiceRequest<T>, DBProstError> {
        T::decode(self.input.as_ref())
            .map(|v|
                ServiceRequest {
                    uri: self.uri.clone(),
                    method: self.method.clone(),
                    version: self.version,
                    headers: self.headers.clone(),
                    input:v,
                })
            .map_err(|e|
                DBProstError::AfterBodyError {
                    body: self.input.clone(),
                    method: Some(self.method.clone()),
                    version: self.version,
                    headers: self.headers.clone(),
                    status: None,
                    err: Box::new(DBProstError::ProstDecodeError(e)),
                })
    }
}


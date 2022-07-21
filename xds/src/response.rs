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

use hyper::{body, Body, Response, Version, header, StatusCode};
use prost::{Message};
use crate::error::*;


/// A response with HTTP info and a serialized output object
#[derive(Debug)]
pub struct ServiceResponse<T> {
    /// The HTTP version
    pub version: Version,
    /// The set of headers
    ///
    /// Should always at least have `Content-Type`. Servers will override `Content-Length` on serialization.
    pub headers: header::HeaderMap,
    /// The status code
    pub status: StatusCode,
    /// The serialized output object
    pub output: T,
}

impl<T> ServiceResponse<T> {
    /// Create new service request with the given input object
    ///
    /// This automatically sets the `Content-Type` header as `application/protobuf`.
    pub fn new(output: T) -> ServiceResponse<T> {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "application/protobuf".parse().unwrap());

        ServiceResponse {
            version: Version::default(),
            headers,
            status: StatusCode::OK,
            output,
        }
    }

    /// Copy this response with a different output value
    pub fn clone_with_output<U>(&self, output: U) -> ServiceResponse<U> {
        ServiceResponse { version: self.version, headers: self.headers.clone(), status: self.status, output }
    }
}

// impl<T: Message + Default + 'static> From<T> for ServiceResponse<T> {
//     fn from(v: T) -> ServiceResponse<T> { ServiceResponse::new(v) }
// }

impl ServiceResponse<Vec<u8>> {
    /// Turn a hyper response to a boxed future of a byte-array service response
    pub async fn try_from_hyper(resp: Response<Body>) -> Result<ServiceResponse<Vec<u8>>, DBProstError> {
        let version = resp.version();
        let headers = resp.headers().clone();
        let status = resp.status().clone();
        body::to_bytes(resp.into_body()).await
            .map_err(DBProstError::HyperError)
            .map(move |body| ServiceResponse { version, headers, status, output: body.to_vec() })
    }

    /// Turn a byte-array service response into a hyper response
    pub fn into_hyper(&self) -> Response<Body> {
        let mut resp = Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(self.output.clone())).unwrap();

        let header_mut = resp.headers_mut();
        header_mut.clone_from(&self.headers);
        let lenth_value = header::HeaderValue::from(self.output.len() as u64);
        header_mut.insert(header::CONTENT_LENGTH, lenth_value);
        resp
    }

    /// Serialize the byte-array service response into a protobuf service response
    pub fn try_decode<T: Message + Default + 'static>(&self) -> Result<ServiceResponse<T>, DBProstError> {
        if self.status.is_success() {
            T::decode(self.output.as_ref())
                .map(|v|
                    ServiceResponse {
                        version: self.version,
                        headers: self.headers.clone(),
                        status: self.status.clone(),
                        output: v,
                    })
                .map_err(|e|
                    DBProstError::AfterBodyError {
                        body: self.output.clone(),
                        method: None,
                        version: self.version,
                        headers: self.headers.clone(),
                        status: Some(self.status),
                        err: Box::new(DBProstError::ProstDecodeError(e)),
                    })
        } else {
            let err= DBError::new(self.status.clone(), "internal_err", "Internal Error");
            Err(DBProstError::DBWrapError(err))
        }
    }
}

impl<T: Message + Default + 'static> ServiceResponse<T> {
    pub fn try_encode(&self) -> Result<ServiceResponse<Vec<u8>>, DBProstError> {
        let mut body = Vec::new();
        self.output.encode(&mut body)
            .map(|v|
                ServiceResponse {
                    version: self.version,
                    headers: self.headers.clone(),
                    status: self.status.clone(),
                    output: body,
                })
            .map_err(|e| DBProstError::ProstEncodeError(e))
    }

    /// Turn a hyper response into a protobuf service response
    pub async fn decode_response(resp: Response<Body>) -> Result<ServiceResponse<T>, DBProstError> {
        ServiceResponse::try_from_hyper(resp).await.and_then(|v| v.try_decode())
    }

    /// Turn a protobuf service response into a hyper response
    pub fn into_encoded_hyper(&self) -> Result<Response<Body>, DBProstError> {
        self.try_encode().map(|v| v.into_hyper())
    }
}


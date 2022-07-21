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

use serde_json;
use prost::{DecodeError, EncodeError};
use hyper::{Body, Response, Method, Version, header, StatusCode};
use crate::response::ServiceResponse;


/// A JSON-serializable DBError error
#[derive(Debug)]
pub struct DBError {
    pub status: StatusCode,
    pub error_type: String,
    pub msg: String,
    pub meta: Option<serde_json::Value>,
}

impl DBError {
    /// Create a DBError error with no meta
    pub fn new(status: StatusCode, error_type: &str, msg: &str) -> DBError {
        DBError::new_meta(status, error_type, msg, None)
    }

    /// Create a DBError error with optional meta
    pub fn new_meta(status: StatusCode, error_type: &str, msg: &str, meta: Option<serde_json::Value>) -> DBError {
        DBError { status, error_type: error_type.to_string(), msg: msg.to_string(), meta }
    }

    /// Create a byte-array service response for this error and the given status code
    pub fn to_resp_raw(&self) -> ServiceResponse<Vec<u8>> {
        let output = self.to_json_bytes().unwrap_or_else(|_| "{}".as_bytes().to_vec());
        let mut headers = header::HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "json".parse().unwrap());
        let lenth_value = header::HeaderValue::from(output.len() as u64);
        headers.insert(header::CONTENT_LENGTH, lenth_value);

        ServiceResponse {
            version: Version::default(),
            headers,
            status: self.status,
            output,
        }
    }

    /// Create a hyper response for this error and the given status code
    pub fn to_hyper_resp(&self) -> Response<Body> {
        let body = self.to_json_bytes().unwrap_or_else(|_| "{}".as_bytes().to_vec());

        let mut resp = Response::builder()
            .status(self.status)
            .body(Body::from(body.clone())).unwrap();

        let header_mut = resp.headers_mut();
        header_mut.insert(header::CONTENT_TYPE, "json".parse().unwrap());
        let lenth_value = header::HeaderValue::from(body.len() as u64);
        header_mut.insert(header::CONTENT_LENGTH, lenth_value);
        resp
    }

    /// Create error from Serde JSON value
    pub fn from_json(status: StatusCode, json: serde_json::Value) -> DBError {
        let error_type = json["error_type"].as_str();
        DBError {
            status,
            error_type: error_type.unwrap_or("<no code>").to_string(),
            msg: json["msg"].as_str().unwrap_or("<no message>").to_string(),
            // Put the whole thing as meta if there was no type
            meta: if error_type.is_some() { json.get("meta").map(|v| v.clone()) } else { Some(json.clone()) },
        }
    }

    /// Create error from byte array
    pub fn from_json_bytes(status: StatusCode, json: &[u8]) -> serde_json::Result<DBError> {
        serde_json::from_slice(json).map(|v| DBError::from_json(status, v))
    }

    /// Create Serde JSON value from error
    pub fn to_json(&self) -> serde_json::Value {
        let mut props = serde_json::map::Map::new();
        props.insert("error_type".to_string(), serde_json::Value::String(self.error_type.clone()));
        props.insert("msg".to_string(), serde_json::Value::String(self.msg.clone()));
        if let Some(ref meta) = self.meta { props.insert("meta".to_string(), meta.clone()); }
        serde_json::Value::Object(props)
    }

    /// Create byte array from error
    pub fn to_json_bytes(&self) -> serde_json::Result<Vec<u8>> {
        serde_json::to_vec(&self.to_json())
    }
}

impl From<DBError> for DBProstError {
    fn from(v: DBError) -> DBProstError { DBProstError::DBWrapError(v) }
}





/// An error that can occur during a call to a  service
#[derive(Debug)]
pub enum DBProstError {
    /// A standard DBError error with a type, message, and some metadata
    DBWrapError(DBError),
    /// An error when trying to decode JSON into an error or object
    JsonDecodeError(serde_json::Error),
    /// An error when trying to encode a protobuf object
    ProstEncodeError(EncodeError),
    /// An error when trying to decode a protobuf object
    ProstDecodeError(DecodeError),
    /// A generic hyper error
    HyperError(hyper::Error),
    /// A wrapper for any of the other `DBProstError`s that also includes request/response info
    AfterBodyError {
        /// The request or response's raw body before the error happened
        body: Vec<u8>,
        /// The request method, only present for server errors
        method: Option<Method>,
        /// The request or response's HTTP version
        version: Version,
        /// The request or response's headers
        headers: header::HeaderMap,
        /// The response status, only present for client errors
        status: Option<StatusCode>,
        /// The underlying error
        err: Box<DBProstError>,
    },
}

impl DBProstError {
    /// This same error, or the underlying error if it is an `AfterBodyError`
    pub fn root_err(self) -> DBProstError {
        match self {
            DBProstError::AfterBodyError { err, .. } => err.root_err(),
            _ => self
        }
    }
}
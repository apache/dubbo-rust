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

pub mod client;
pub mod server;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone)]
pub struct Header {
    pub jsonrpc: Option<String>,
    pub id: Option<String>,
    pub method: Option<String>,
}

pub struct Request {
    pub header: Header,
    pub params: serde_json::Value,
}
impl Request {
    fn from_slice(slice: Vec<u8>) -> Result<Self, serde_json::Error> {
        let pre: serde_json::map::Map<String, serde_json::Value> = serde_json::from_slice(&slice)?;

        let header = Header {
            jsonrpc: Self::get_json_value(pre.get("jsonrpc"))?,
            id: Self::get_json_value(pre.get("id"))?,
            method: Self::get_json_value(pre.get("method"))?,
        };

        let params = serde_json::from_value(pre.get("params").unwrap_or(&json!({})).clone())?;

        Ok(Self { header, params })
    }

    fn get_json_value<T: DeserializeOwned>(
        value: Option<&serde_json::Value>,
    ) -> Result<Option<T>, serde_json::Error> {
        match value {
            None => Ok(None),
            Some(v) => {
                let ret: T = serde_json::from_value(v.clone())?;

                Ok(Some(ret))
            }
        }
    }

    pub fn new<T: Serialize>(method_name: &str, req: T) -> Result<Self, serde_json::Error> {
        Ok(Self {
            header: Header {
                jsonrpc: Some("2.0".to_string()),
                id: Some("1".to_string()),
                method: Some(method_name.to_string()),
            },
            params: serde_json::to_value(&req)?,
        })
    }

    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        Ok(serde_json::to_string(&json!({
            "jsonrpc": self.header.jsonrpc,
            "id": self.header.id,
            "method": self.header.method,
            "params": self.params
        }))?)
    }
}

pub struct Response {
    header: Header,
    error: Option<Error>,
    result: Option<serde_json::Value>,
}

impl Response {
    pub fn from_request<B: Serialize>(
        request: &Request,
        result: B,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let p = serde_json::to_value(result)?;
        Ok(Self {
            header: request.header.clone(),
            error: None,
            result: Some(p),
        })
    }

    pub fn from_request_error(request: &Request, error: Error) -> Self {
        Self {
            header: request.header.clone(),
            error: Some(error),
            result: None,
        }
    }

    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        let ret = json!({
            "jsonrpc": self.header.jsonrpc,
            "id": self.header.id,
            "method": self.header.method,
            "result": self.result,
            "error": self.error
        });

        Ok(serde_json::to_string(&ret)?)
    }

    pub fn from_slice(slice: &Vec<u8>) -> Result<Self, serde_json::Error> {
        let pre: serde_json::map::Map<String, serde_json::Value> = serde_json::from_slice(&slice)?;

        let header = Header {
            jsonrpc: Self::get_json_value(pre.get("jsonrpc"))?,
            id: Self::get_json_value(pre.get("id"))?,
            method: Self::get_json_value(pre.get("method"))?,
        };

        let result = serde_json::from_value(pre.get("result").unwrap_or(&json!({})).clone())?;
        let error = serde_json::from_value(pre.get("error").unwrap_or(&json!({})).clone())?;
        Ok(Self {
            header,
            result,
            error,
        })
    }

    pub fn get_body<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        Ok(serde_json::from_value(
            self.result.as_ref().unwrap_or(&json!({})).clone(),
        )?)
    }

    fn get_json_value<T: DeserializeOwned>(
        value: Option<&serde_json::Value>,
    ) -> Result<Option<T>, serde_json::Error> {
        match value {
            None => Ok(None),
            Some(v) => {
                let ret: T = serde_json::from_value(v.clone())?;

                Ok(Some(ret))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Error {
    pub code: i64,
    pub message: String,
}

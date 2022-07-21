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

use hyper::client::conn::Builder;
use hyper::client::connect::HttpConnector;
use hyper::client::service::Connect;
use hyper::service::Service;
use hyper::{Body, Request, Response};

use crate::protocol::message::{
    Message as DubboMessage,
    RpcxMessage,
    Metadata,
    MessageType,
    CompressType,
    SerializeType
};

use std::collections::HashMap;
use prost::{Message};

use crate::request::ServiceRequest;
use crate::response::ServiceResponse;
use crate::error::*;


pub struct RpcClient {
    addr: String
}

impl RpcClient {
    pub fn new(addr: String) -> RpcClient {
        RpcClient {
            addr
        }
    }

    /// Invoke the given request for the given path and return a result of Result<ServiceResponse<O>, DBProstError>
    pub async fn request<I, O>(&self, message: I, path: String) -> Result<ServiceResponse<O>, DBProstError>
        where I: Message + Default + 'static,
              O: Message + Default + 'static {

        let url_str = format!("{}/{}", self.addr.as_str(), path.as_str());
        let uri = url_str.parse::<hyper::Uri>().unwrap();

        let req = ServiceRequest::new(message, uri.clone());
        let hyper_req = req.into_encoded_hyper().unwrap();

        let mut connect = Connect::new(HttpConnector::new(), Builder::new());
        let mut send = connect.call(uri.clone()).await.map_err(DBProstError::HyperError).unwrap();
        let hyper_resp = send.call(hyper_req).await.map_err(DBProstError::HyperError).unwrap();
        ServiceResponse::decode_response(hyper_resp).await
    }

    pub async fn call(
        &mut self,
        service_path: String,
        service_method: String,
        metadata: &Metadata,
        payload: Vec<u8>
    ) -> std::result::Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut req = DubboMessage::new();
        req.set_version(0);
        req.set_message_type(MessageType::Request);
        req.set_serialize_type(SerializeType::Protobuf);
        req.set_compress_type(CompressType::Gzip);
        req.service_path = service_path;
        req.service_method = service_method;
        req.payload = payload;

        let mut new_metadata = HashMap::with_capacity(metadata.len());
        for (k, v) in metadata {
            new_metadata.insert(k.clone(), v.clone());
        }
        req.metadata.replace(new_metadata);
        let body_data = req.encode();

        // println!("call, body_data={:?}", body_data);

        let mut mk_svc = Connect::new(HttpConnector::new(), Builder::new());
        let uri = self.addr.parse::<hyper::Uri>().unwrap();
        let mut svc = mk_svc.call(uri.clone()).await?;

        let body = Body::from(body_data);
        let req = Request::get(uri.clone()).body(body)?;
        let res = svc.call(req).await?;

        Ok(res)
    }
}


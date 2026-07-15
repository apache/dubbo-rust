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

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use dubbo_rs_core::{RawMetadata, RawTripleClient, RawUnaryRequest as CoreUnaryRequest};
use napi::{
    bindgen_prelude::{AsyncTask, Buffer},
    Error, Result, Status, Task,
};
use napi_derive::napi;
use tokio::runtime::{Builder, Runtime};

#[napi(object)]
pub struct NativeDubboTransportOptions {
    #[napi(js_name = "baseUrl")]
    pub base_url: Option<String>,
    pub registry: Option<String>,
    pub protocol: Option<String>,
    #[napi(js_name = "timeoutMs")]
    pub timeout_ms: Option<u32>,
    #[napi(js_name = "loadBalance")]
    pub load_balance: Option<String>,
    pub cluster: Option<String>,
    pub group: Option<String>,
    pub version: Option<String>,
}

#[napi(object)]
pub struct NativeUnaryRequest {
    pub service: String,
    pub method: String,
    pub path: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Buffer,
    #[napi(js_name = "timeoutMs")]
    pub timeout_ms: Option<u32>,
    pub group: Option<String>,
    pub version: Option<String>,
}

#[napi(object)]
pub struct NativeUnaryResponse {
    pub code: u32,
    pub message: Option<String>,
    pub headers: HashMap<String, String>,
    pub trailers: HashMap<String, String>,
    pub body: Buffer,
}

#[napi]
pub struct NativeTransport {
    client: Arc<Mutex<RawTripleClient>>,
    runtime: Arc<Runtime>,
}

#[napi]
impl NativeTransport {
    #[napi]
    pub fn unary(&self, request: NativeUnaryRequest) -> AsyncTask<UnaryTask> {
        AsyncTask::new(UnaryTask {
            client: Arc::clone(&self.client),
            runtime: Arc::clone(&self.runtime),
            request,
        })
    }

    #[napi]
    pub fn close(&self) -> Result<()> {
        Ok(())
    }
}

#[napi]
pub fn create_native_transport(options: NativeDubboTransportOptions) -> Result<NativeTransport> {
    if options
        .protocol
        .as_deref()
        .is_some_and(|protocol| protocol != "triple")
    {
        return Err(Error::new(
            Status::InvalidArg,
            "only the triple protocol is supported by dubbo-node-native".to_string(),
        ));
    }

    if options.registry.is_some() {
        return Err(Error::new(
            Status::InvalidArg,
            "registry URLs are not supported by dubbo-node-native yet; use baseUrl for direct Triple calls".to_string(),
        ));
    }

    let base_url = options.base_url.ok_or_else(|| {
        Error::new(
            Status::InvalidArg,
            "baseUrl is required until registry support lands".to_string(),
        )
    })?;

    let runtime = Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|err| Error::new(Status::GenericFailure, err.to_string()))?;

    Ok(NativeTransport {
        client: Arc::new(Mutex::new(RawTripleClient::from_static(&base_url))),
        runtime: Arc::new(runtime),
    })
}

pub struct UnaryTask {
    client: Arc<Mutex<RawTripleClient>>,
    runtime: Arc<Runtime>,
    request: NativeUnaryRequest,
}

impl Task for UnaryTask {
    type Output = NativeUnaryResponse;
    type JsValue = NativeUnaryResponse;

    fn compute(&mut self) -> Result<Self::Output> {
        let request = std::mem::replace(
            &mut self.request,
            NativeUnaryRequest {
                service: String::new(),
                method: String::new(),
                path: String::new(),
                headers: None,
                body: Buffer::from(Vec::new()),
                timeout_ms: None,
                group: None,
                version: None,
            },
        );
        let mut client = self
            .client
            .lock()
            .map_err(|_| Error::new(Status::GenericFailure, "native transport lock poisoned"))?;

        let response = self.runtime.block_on(client.unary(core_request(request)));

        response
            .map(native_response)
            .map_err(|err| Error::new(Status::GenericFailure, err.to_string()))
    }

    fn resolve(&mut self, _env: napi::Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}

fn core_request(request: NativeUnaryRequest) -> CoreUnaryRequest {
    let mut metadata = request
        .headers
        .unwrap_or_default()
        .into_iter()
        .fold(RawMetadata::new(), |metadata, (key, value)| {
            metadata.insert(key, value)
        });

    if let Some(group) = request.group {
        metadata = metadata.insert("tri-service-group", group);
    }
    if let Some(version) = request.version {
        metadata = metadata.insert("tri-service-version", version);
    }

    CoreUnaryRequest {
        service: request.service,
        method: request.method,
        path: request.path,
        metadata,
        body: Bytes::from(request.body.to_vec()),
    }
}

fn native_response(response: dubbo_rs_core::RawUnaryResponse) -> NativeUnaryResponse {
    NativeUnaryResponse {
        code: 0,
        message: None,
        headers: metadata_entries(response.metadata),
        trailers: HashMap::new(),
        body: Buffer::from(response.body.to_vec()),
    }
}

fn metadata_entries(metadata: RawMetadata) -> HashMap<String, String> {
    metadata.entries.into_iter().collect()
}

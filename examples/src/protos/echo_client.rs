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

use super::echo_server::{HelloReply, HelloRequest};

use triple::client::TripleClient;
use triple::codec::serde_codec::SerdeCodec;
use triple::invocation::*;
use triple::server::Decoding;

pub struct EchoClient {
    inner: TripleClient,
    uri: String,
}

impl Default for EchoClient {
    fn default() -> Self {
        Self::new()
    }
}

impl EchoClient {
    pub fn new() -> Self {
        Self {
            inner: TripleClient::new(),
            uri: "".to_string(),
        }
    }

    pub fn with_uri(mut self, uri: String) -> Self {
        self.uri = uri.clone();
        self.inner = self.inner.with_host(uri);
        self
    }

    pub async fn bidirectional_streaming_echo(
        &mut self,
        req: impl IntoStreamingRequest<Message = HelloRequest>,
    ) -> Result<Response<Decoding<HelloReply>>, triple::status::Status> {
        let codec = SerdeCodec::<HelloRequest, HelloReply>::default();
        self.inner
            .bidi_streaming(
                req,
                codec,
                http::uri::PathAndQuery::from_static("/echo/bidi_stream"),
            )
            .await
    }

    pub async fn say_hello(
        &mut self,
        req: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, triple::status::Status> {
        let codec = SerdeCodec::<HelloRequest, HelloReply>::default();
        self.inner
            .unary(
                req,
                codec,
                http::uri::PathAndQuery::from_static("/echo/hello"),
            )
            .await
    }

    pub async fn client_streaming(
        &mut self,
        req: impl IntoStreamingRequest<Message = HelloRequest>,
    ) -> Result<Response<HelloReply>, triple::status::Status> {
        let codec = SerdeCodec::<HelloRequest, HelloReply>::default();
        self.inner
            .client_streaming(
                req,
                codec,
                http::uri::PathAndQuery::from_static("/echo/client_streaming"),
            )
            .await
    }

    pub async fn server_streaming(
        &mut self,
        req: Request<HelloRequest>,
    ) -> Result<Response<Decoding<HelloReply>>, triple::status::Status> {
        let codec = SerdeCodec::<HelloRequest, HelloReply>::default();
        self.inner
            .server_streaming(
                req,
                codec,
                http::uri::PathAndQuery::from_static("/echo/server_streaming"),
            )
            .await
    }
}

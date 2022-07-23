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

use futures_util::{future, stream, StreamExt, TryStreamExt};
use http_body::Body;

use crate::codec::Codec;
use crate::invocation::Request;
use crate::server::encode::encode_server;
use crate::server::service::{StreamingSvc, UnaryService};
use crate::server::Streaming;
use crate::BoxBody;
use config::BusinessConfig;

pub struct TripleServer<T> {
    codec: T,
}

impl<T> TripleServer<T> {
    pub fn new(codec: T) -> Self {
        Self { codec }
    }
}

impl<T> TripleServer<T>
where
    T: Codec,
{
    pub async fn bidi_streaming<S, B>(
        &mut self,
        mut service: S,
        req: http::Request<B>,
    ) -> http::Response<BoxBody>
    where
        S: StreamingSvc<T::Decode, Response = T::Encode>,
        S::ResponseStream: Send + 'static,
        B: Body + Send + 'static,
        B::Error: Into<crate::Error> + Send,
    {
        let req_stream = req.map(|body| Streaming::new(body, self.codec.decoder()));

        let resp = service.call(Request::from_http(req_stream)).await;

        let (mut parts, resp_body) = resp.unwrap().into_http().into_parts();
        let resp_body = encode_server(self.codec.encoder(), resp_body);

        parts.headers.insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("application/grpc"),
        );
        parts.status = http::StatusCode::OK;
        http::Response::from_parts(parts, BoxBody::new(resp_body))
    }

    pub async fn unary<S, B>(
        &mut self,
        mut service: S,
        req: http::Request<B>,
    ) -> http::Response<BoxBody>
    where
        S: UnaryService<T::Decode, Response = T::Encode>,
        B: Body + Send + 'static,
        B::Error: Into<crate::Error> + Send,
    {
        let req_stream = req.map(|body| Streaming::new(body, self.codec.decoder()));
        let (parts, mut body) = Request::from_http(req_stream).into_parts();
        let msg = body
            .try_next()
            .await
            .unwrap()
            .ok_or_else(|| tonic::Status::new(tonic::Code::Unknown, "request wrong"));

        let resp = service.call(Request::from_parts(parts, msg.unwrap())).await;

        let (mut parts, resp_body) = resp.unwrap().into_http().into_parts();
        let resp_body = encode_server(
            self.codec.encoder(),
            stream::once(future::ready(resp_body)).map(Ok).into_stream(),
        );

        parts.headers.insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("application/grpc"),
        );
        parts.status = http::StatusCode::OK;
        http::Response::from_parts(parts, BoxBody::new(resp_body))
    }
}

impl<T> BusinessConfig for TripleServer<T> {
    fn init() -> Self {
        todo!()
    }

    fn load() -> Result<(), std::convert::Infallible> {
        todo!()
    }
}

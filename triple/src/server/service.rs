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

use super::decode::Decoding;
use crate::invocation::{Request, Response};
use futures_util::{Future, Stream};
use tower_service::Service;

pub trait StreamingSvc<R> {
    // proto response
    type Response;
    // the stream of proto message
    type ResponseStream: Stream<Item = Result<Self::Response, tonic::Status>>;
    // future of stream of proto message
    type Future: Future<Output = Result<Response<Self::ResponseStream>, tonic::Status>>;

    fn call(&mut self, req: Request<Decoding<R>>) -> Self::Future;
}

impl<T, S, M1, M2> StreamingSvc<M1> for T
where
    T: Service<Request<Decoding<M1>>, Response = Response<S>, Error = tonic::Status>,
    S: Stream<Item = Result<M2, tonic::Status>>,
{
    type Response = M2;

    type ResponseStream = S;

    type Future = T::Future;

    fn call(&mut self, req: Request<Decoding<M1>>) -> Self::Future {
        Service::call(self, req)
    }
}

pub trait UnaryService<R> {
    type Response;
    type Future: Future<Output = Result<Response<Self::Response>, tonic::Status>>;

    fn call(&mut self, req: Request<R>) -> Self::Future;
}

impl<T, M1, M2> UnaryService<M1> for T
where
    T: Service<Request<M1>, Response = Response<M2>, Error = tonic::Status>,
{
    type Response = M2;

    type Future = T::Future;

    fn call(&mut self, req: Request<M1>) -> Self::Future {
        T::call(self, req)
    }
}

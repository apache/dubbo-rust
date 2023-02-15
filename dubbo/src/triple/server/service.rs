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

use futures_util::{Future, Stream};
use tower_service::Service;

use crate::{
    invocation::{Request, Response},
    triple::decode::Decoding,
};

pub trait StreamingSvc<R> {
    type Response;

    type ResponseStream: Stream<Item = Result<Self::Response, crate::status::Status>>;

    type Future: Future<Output = Result<Response<Self::ResponseStream>, crate::status::Status>>;

    fn call(&mut self, req: Request<Decoding<R>>) -> Self::Future;
}

impl<T, S, M1, M2> StreamingSvc<M1> for T
where
    T: Service<Request<Decoding<M1>>, Response = Response<S>, Error = crate::status::Status>,
    S: Stream<Item = Result<M2, crate::status::Status>>,
{
    type Response = M2;

    type ResponseStream = S;

    type Future = T::Future;

    fn call(&mut self, req: Request<Decoding<M1>>) -> Self::Future {
        Service::call(self, req)
    }
}

pub trait UnarySvc<R> {
    type Response;
    type Future: Future<Output = Result<Response<Self::Response>, crate::status::Status>>;

    fn call(&mut self, req: Request<R>) -> Self::Future;
}

impl<T, M1, M2> UnarySvc<M1> for T
where
    T: Service<Request<M1>, Response = Response<M2>, Error = crate::status::Status>,
{
    type Response = M2;

    type Future = T::Future;

    fn call(&mut self, req: Request<M1>) -> Self::Future {
        T::call(self, req)
    }
}

pub trait ClientStreamingSvc<R> {
    type Response;
    type Future: Future<Output = Result<Response<Self::Response>, crate::status::Status>>;

    fn call(&mut self, req: Request<Decoding<R>>) -> Self::Future;
}

impl<T, M1, M2> ClientStreamingSvc<M1> for T
where
    T: Service<Request<Decoding<M1>>, Response = Response<M2>, Error = crate::status::Status>,
{
    type Response = M2;

    type Future = T::Future;

    fn call(&mut self, req: Request<Decoding<M1>>) -> Self::Future {
        T::call(self, req)
    }
}

pub trait ServerStreamingSvc<R> {
    type Response;

    type ResponseStream: Stream<Item = Result<Self::Response, crate::status::Status>>;

    type Future: Future<Output = Result<Response<Self::ResponseStream>, crate::status::Status>>;

    fn call(&mut self, req: Request<R>) -> Self::Future;
}

impl<T, S, M1, M2> ServerStreamingSvc<M1> for T
where
    T: Service<Request<M1>, Response = Response<S>, Error = crate::status::Status>,
    S: Stream<Item = Result<M2, crate::status::Status>>,
{
    type Response = M2;

    type ResponseStream = S;

    type Future = T::Future;

    fn call(&mut self, req: Request<M1>) -> Self::Future {
        Service::call(self, req)
    }
}

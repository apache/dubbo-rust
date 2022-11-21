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
//! Implement the [`ClusterFilter`].

use tower_service::Service;

use core::fmt;
use std::task::Poll;

use bytes::Bytes;
use futures::Future;
use pin_project::pin_project;
use tower::Layer;

use crate::invocation::Metadata;
use crate::invocation::Request;
use crate::{boxed, status::Status, BoxBody, Error};

use super::ClusterFilter;

impl<F> ClusterFilter for F
where
    F: FnMut(Request<()>) -> Result<Request<()>, Status>,
{
    fn call(&mut self, req: Request<()>) -> Result<Request<()>, crate::status::Status> {
        self(req)
    }
}

/// Create a [`ClusterFilterLayer`].
///
/// See [`ClusterFilter`] for more details.
pub fn cluster_filter<F>(filter: F) -> ClusterFilterLayer<F> {
    ClusterFilterLayer { filter }
}

/// A [`ClusterFilter`] can be used as a [`Layer`],
/// is created by calling [`cluster_filter`].
///
/// See [`ClusterFilterService`] for more details.
#[derive(Debug, Clone, Copy)]
pub struct ClusterFilterLayer<F> {
    filter: F,
}

impl<S, F: Clone> Layer<S> for ClusterFilterLayer<F> {
    type Service = ClusterFilterService<S, F>;

    fn layer(&self, inner: S) -> Self::Service {
        ClusterFilterService::new(inner, self.filter.clone())
    }
}

/// The service will call the filter `CF` to preprocess the HTTP Request, and then pass the request to the service `S`.
///
/// See [`ClusterFilter`] for more details.
#[derive(Clone, Copy)]
pub struct ClusterFilterService<S, CF> {
    inner: S,
    filter: CF,
}

impl<S, CF> ClusterFilterService<S, CF> {
    fn new(inner: S, filter: CF) -> Self {
        Self { inner, filter }
    }
}

impl<S, CF> fmt::Debug for ClusterFilterService<S, CF>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClusterFilterService")
            .field("inner", &self.inner)
            .field("filter", &format_args!("{}", std::any::type_name::<CF>()))
            .finish()
    }
}

/// A service which call [`ClusterFilter`] and then pass the result to the inner [`Service`].
impl<S, CF, ReqBody, RespBody> Service<http::Request<ReqBody>> for ClusterFilterService<S, CF>
where
    CF: ClusterFilter,
    S: Service<http::Request<ReqBody>, Response = http::Response<RespBody>>,
    S::Error: Into<Error>,
    ReqBody: http_body::Body<Data = Bytes> + Send + 'static,
    ReqBody::Error: Into<Error>,
    RespBody: http_body::Body<Data = Bytes> + Send + 'static,
    RespBody::Error: Into<Error>,
{
    type Response = http::Response<BoxBody>;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        // It's a bad practice to modify the HTTP Request's body, so we extract it from the original
        // request and recreate a Dubbo's HTTP Request, which can avoid exposing the message to the
        // [`ClusterFilter`]. Dubbo's [`invocation::Request`] don't preserve the URI, Method and Version of the
        // original HTTP Request, so we extract them and set them back later when recreate the HTTP Request.
        let uri = req.uri().clone();
        let method = req.method().clone();
        let version = req.version();
        let (parts, msg) = req.into_parts();

        let req = Request::from_parts(Metadata::from_headers(parts.headers), ());
        match self.filter.call(req) {
            Ok(req) => {
                let (metadata, _) = req.into_parts();
                let req = Request::from_parts(metadata, msg);
                let http_req = req.into_http(uri, method, version);
                ResponseFuture::furure(self.inner.call(http_req))
            }
            Err(status) => ResponseFuture::status(status),
        }
    }
}

/// A general Future for [`ClusterFilterService`].
#[pin_project]
#[derive(Debug)]
pub struct ResponseFuture<F> {
    #[pin]
    kind: Kind<F>,
}

#[pin_project(project = KindProj)]
#[derive(Debug)]
enum Kind<F> {
    Future(#[pin] F),
    Status(Option<Status>),
}

impl<F> ResponseFuture<F> {
    fn furure(future: F) -> Self {
        Self {
            kind: Kind::Future(future),
        }
    }

    fn status(status: Status) -> Self {
        Self {
            kind: Kind::Status(Some(status)),
        }
    }
}

impl<F, RespBody, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<http::Response<RespBody>, E>>,
    E: Into<Error>,
    RespBody: http_body::Body<Data = Bytes> + Send + 'static,
    RespBody::Error: Into<Error>,
{
    type Output = Result<http::Response<BoxBody>, E>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.project().kind.project() {
            KindProj::Future(fut) => fut
                .poll(cx)
                .map(|result| result.map(|resp| resp.map(boxed))),
            KindProj::Status(status) => {
                Poll::Ready(Ok(status.take().unwrap().to_http().map(boxed)))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use tower::service_fn;
    use tower::ServiceBuilder;
    use tower::ServiceExt;

    use crate::empty_body;
    use crate::status::Code;
    use crate::status::Status;
    use crate::BoxBody;

    use super::*;

    const USER_AGENT: &str = "user-agent";
    const USER_AGENT_VAL: &str = "dubbo-test";

    #[derive(Clone, Copy)]
    struct MyFilter;

    impl ClusterFilter for MyFilter {
        fn call(&mut self, req: Request<()>) -> Result<Request<()>, crate::status::Status> {
            assert_eq!(
                req.metadata()
                    .get_ref()
                    .get(USER_AGENT)
                    .expect("missing user-agent."),
                USER_AGENT_VAL
            );
            Ok::<_, Status>(req)
        }
    }

    #[tokio::test]
    async fn doesnt_change_anything() {
        let svc = service_fn(|req: http::Request<BoxBody>| async move {
            assert_eq!(
                req.headers().get(USER_AGENT).expect("missing user-agent."),
                USER_AGENT_VAL
            );
            Ok::<_, Status>(http::Response::new(empty_body()))
        });
        let svc = ClusterFilterService::new(svc, MyFilter);
        let req = http::Request::builder()
            .header(USER_AGENT, USER_AGENT_VAL)
            .body(empty_body())
            .unwrap();
        svc.oneshot(req).await.unwrap();
    }

    #[tokio::test]
    async fn add_cluster_filter_to_service() {
        let svc = service_fn(|req: http::Request<BoxBody>| async move {
            assert_eq!(
                req.headers().get(USER_AGENT).expect("missing user-agent."),
                USER_AGENT_VAL
            );
            Ok::<_, Status>(http::Response::new(empty_body()))
        });
        let svc = ServiceBuilder::new()
            .layer(cluster_filter(MyFilter))
            .service(svc);
        let req = http::Request::builder()
            .header(USER_AGENT, USER_AGENT_VAL)
            .body(empty_body())
            .unwrap();
        svc.oneshot(req).await.unwrap();
    }

    #[tokio::test]
    async fn handle_status_as_response() {
        let msg = "PermissionDenied from ClusterFilter.";
        let expected = Status::new(Code::PermissionDenied, msg).to_http();

        let svc = service_fn(|_: http::Request<BoxBody>| async move {
            Ok::<_, Status>(http::Response::new(empty_body()))
        });
        let svc = ClusterFilterService::new(svc, |_: Request<()>| -> Result<Request<()>, Status> {
            Err(Status::new(Code::PermissionDenied, msg))
        });
        let resp = svc
            .oneshot(http::Request::builder().body(empty_body()).unwrap())
            .await
            .unwrap();

        assert_eq!(resp.headers(), expected.headers());
        assert_eq!(resp.status(), expected.status());
        assert_eq!(resp.version(), expected.version());
    }
}

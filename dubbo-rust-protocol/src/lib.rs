use http::request::Request as HttpRequest;
use http::response::Response as HttpResponse;
use hyper::server::conn::Connection;
use hyper::Body;
use pin_project_lite::pin_project;
use std::any::Any;
use std::pin::Pin;
use std::task::Poll;
use std::{convert::Infallible, future::Future};
use tokio::io::{AsyncRead, AsyncWrite};

pub mod jsonrpc;
pub mod triple;

pub(crate) fn wrap_future<F, R, E>(fut: F) -> SrvFut<R, E>
where
    F: Future<Output = Result<R, E>> + Send + 'static,
{
    Box::pin(fut)
}

type SrvFut<R, E> = Pin<Box<dyn Future<Output = Result<R, E>> + Send + 'static>>;

pub trait NamedService {
    const SERVICE_NAME: &'static str;
}

pin_project! {
    struct OneConnection<IO,S>
    where S: tower::Service<HttpRequest<Body>,Response = HttpResponse<Body>,Error = StdError, Future = SrvFut<HttpResponse<Body>,StdError>>
    {
        #[pin]
        connection: Connection<IO,S>
    }
}

type StdError = Box<dyn std::error::Error + Send + Sync + 'static>;

impl<IO, S> Future for OneConnection<IO, S>
where
    S: tower::Service<
            HttpRequest<Body>,
            Response = HttpResponse<Body>,
            Error = StdError,
            Future = SrvFut<HttpResponse<Body>, StdError>,
        > + Unpin,
    IO: AsyncRead + AsyncWrite + Unpin,
{
    type Output = Result<(), hyper::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        self.project().connection.poll_without_shutdown(cx)
    }
}

// codec define
pub trait Codec {
    fn decode<T>(req: HttpRequest<Body>) -> Result<T, Infallible>;
    fn encode<T>(req: T) -> Result<HttpResponse<Body>, Infallible>;
}

pub struct TripleRequest {
    inner: Box<dyn Any + 'static + Send>,
}

impl TripleRequest {
    pub fn from<T: 'static + Send>(typ: T) -> Self {
        Self {
            inner: Box::new(typ),
        }
    }

    pub fn downcast<T: 'static>(self) -> Box<T> {
        self.inner.downcast().unwrap()
    }
}

pub struct TripleResponse {
    inner: Box<dyn Any + 'static + Send>,
}

impl TripleResponse {
    pub fn from<T: 'static + Send>(typ: T) -> Self {
        Self {
            inner: Box::new(typ),
        }
    }

    pub fn downcast<T: 'static>(self) -> Box<T> {
        self.inner.downcast().unwrap()
    }
}

pub trait Api: Clone + Send + 'static + Sync {
    fn call_method(
        &self,
        method: &str,
        request: TripleRequest,
    ) -> Pin<Box<dyn Future<Output = Result<TripleResponse, ()>> + Send + 'static>>;

    fn decode(&self, method: &str, req: HttpRequest<Body>) -> Result<TripleRequest, ()>;

    fn encode(&self, method: &str, resp: TripleResponse) -> Result<HttpResponse<Body>, ()>;
}

use hyper::Body;
use hyper::Request as HttpRequest;
use hyper::Response as HttpResponse;
use tower::Service;

use crate::Api;
use crate::SrvFut;
use crate::StdError;

pub struct TripleService<S> {
    inner: S,
}

impl<S: Api> TripleService<S> {
    pub fn new(srv: S) -> Self {
        Self { inner: srv }
    }
}

impl<S: Api> Service<HttpRequest<Body>> for TripleService<S> {
    type Response = HttpResponse<Body>;

    type Error = StdError;

    type Future = SrvFut<Self::Response, Self::Error>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: HttpRequest<Body>) -> Self::Future {
        let method_not_found = || {
            Box::pin(async move {
                Ok(HttpResponse::builder()
                    .status(404)
                    .body(Body::empty())
                    .unwrap())
            })
        };
        let method_name = req.uri().path().split("/").last();
        if method_name.is_none() {
            return method_not_found();
        }
        let method_name = method_name.unwrap();

        let inner_service = self.inner.clone();
        Box::pin(async move {
            let fail = Ok(HttpResponse::builder()
                .status(500)
                .body(Body::empty())
                .unwrap());

            if let Ok(req) = inner_service.decode(method_name, req) {
                if let Ok(resp) = inner_service.call_method(method_name, req).await {
                    if let Ok(http_resp) = inner_service.encode(method_name, resp) {
                        Ok(HttpResponse::builder()
                            .status(200)
                            .body(Body::empty())
                            .unwrap())
                    } else {
                        return fail;
                    }
                } else {
                    return fail;
                }
            } else {
                return fail;
            }
        })
    }
}

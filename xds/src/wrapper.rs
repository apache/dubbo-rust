use std::sync::Arc;
use std::task::Poll;
use futures::future::BoxFuture;

use hyper::{Body, Request, Response, header, StatusCode};

use hyper::client::conn::Builder;
use hyper::client::connect::HttpConnector;
use hyper::client::service::Connect;

use tower::Service;
use prost::{Message};

use crate::request::ServiceRequest;
use crate::response::ServiceResponse;
use crate::util::*;
use crate::error::*;



/// A wrapper for a hyper client
#[derive(Debug)]
pub struct HyperClient {
    /// The hyper client
    /// The root URL without any path attached
    pub root_url: String,
}

impl HyperClient {
    /// Create a new client wrapper for the given client and root using protobuf
    pub fn new(root_url: &str) -> HyperClient {
        HyperClient {
            root_url: root_url.trim_end_matches('/').to_string(),
        }
    }

    // Invoke the given request for the given path and return a boxed future result
    pub async fn request<I, O>(&self, path: &str, req: ServiceRequest<I>) -> Result<ServiceResponse<O>, DBProstError>
        where I: Message + Default + 'static, O: Message + Default + 'static {
        let hyper_req = req.to_hyper_proto().unwrap();
        let mut hyper_connect = Connect::new(HttpConnector::new(), Builder::new());

        let url_string = format!("{}/{}", self.root_url, path.trim_start_matches('/'));
        let uri = url_string.parse::<hyper::Uri>().unwrap();

        let mut req_to_send = hyper_connect.call(uri.clone()).await.map_err(DBProstError::HyperError).unwrap();

        let hyper_resp = req_to_send.call(hyper_req).await.map_err(DBProstError::HyperError).unwrap();
        ServiceResponse::from_hyper_proto(hyper_resp).await
    }
}

/// Service for taking a raw service request and returning a boxed future of a raw service response
pub trait HyperService {
    /// Accept a raw service request and return a boxed future of a raw service response
    fn handle(&self, req: ServiceRequest<Vec<u8>>) -> BoxFutureResp<Vec<u8>>;
}

/// A wrapper for a `HyperService` trait that keeps a `Arc` version of the service
pub struct HyperServer<T> {
    /// The `Arc` version of the service
    ///
    /// Needed because of [hyper Service lifetimes](https://github.com/tokio-rs/tokio-service/issues/9)
    pub service: Arc<T>,
}

impl<T> HyperServer<T> {
    /// Create a new service wrapper for the given impl
    pub fn new(service: T) -> HyperServer<T> { HyperServer { service: Arc::new(service) } }
}

impl<T> Service<Request<Body>> for HyperServer<T>
    where T: 'static + Send + Sync + HyperService
{
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let content_type = req.headers().get(header::CONTENT_TYPE).unwrap().to_str().unwrap();
        if content_type == "application/protobuf" {
            Box::pin(async move {
                let status = StatusCode::UNSUPPORTED_MEDIA_TYPE;
                Ok(DBError::new(status, "bad_content_type", "Content type must be application/protobuf").to_hyper_resp())
            })
        } else {
            let service = self.service.clone();
            Box::pin(async move {
                let request = ServiceRequest::from_hyper_raw(req).await;
                let resp = service.handle(request.unwrap()).await;
                resp.map(|v| v.to_hyper_raw())
                    .or_else(|err| match err.root_err() {
                        DBProstError::ProstDecodeError(_) =>
                            Ok(DBError::new(StatusCode::BAD_REQUEST, "protobuf_decode_err", "Invalid protobuf body").
                                to_hyper_resp()),
                        DBProstError::DBWrapError(err) =>
                            Ok(err.to_hyper_resp()),
                        DBProstError::HyperError(err) =>
                            Err(err),
                        _ => Ok(DBError::new(StatusCode::INTERNAL_SERVER_ERROR, "internal_err", "Internal Error").
                            to_hyper_resp()),
                    })
            })
        }
    }
}
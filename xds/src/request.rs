use hyper::{body, Body, Request, Uri, Method, Version, header};
use prost::{Message};
use crate::util::*;
use crate::error::*;

/// A request with HTTP info and the serialized input object
#[derive(Debug)]
pub struct ServiceRequest<T> {
    /// The URI of the original request
    ///
    /// When using a client, this will be overridden with the proper URI. It is only valuable for servers.
    pub uri: Uri,
    /// The request method; should always be Post
    pub method: Method,
    /// The HTTP version, rarely changed from the default
    pub version: Version,
    /// The set of headers
    ///
    /// Should always at least have `Content-Type`. Clients will override `Content-Length` on serialization.
    pub headers: header::HeaderMap,
    // The serialized request object
    pub input: T,
}

impl<T> ServiceRequest<T> {
    /// Create new service request with the given input object
    ///
    /// This automatically sets the `Content-Type` header as `application/protobuf`.
    pub fn new(input: T) -> ServiceRequest<T> {
        let mut header = header::HeaderMap::new();
        header.insert(header::CONTENT_TYPE, "application/protobuf".parse().unwrap());
        ServiceRequest {
            uri: Default::default(),
            method: Method::POST,
            version: Version::default(),
            headers: header,
            input,
        }
    }

    /// Copy this request with a different input value
    pub fn clone_with_input<U>(&self, input: U) -> ServiceRequest<U> {
        ServiceRequest {
            uri: self.uri.clone(),
            method: self.method.clone(),
            version: self.version,
            headers: self.headers.clone(),
            input,
        }
    }
}

impl<T: Message + Default + 'static> From<T> for ServiceRequest<T> {
    fn from(v: T) -> ServiceRequest<T> { ServiceRequest::new(v) }
}

impl ServiceRequest<Vec<u8>> {
    /// Turn a hyper request to a boxed future of a byte-array service request
    pub fn from_hyper_raw(req: Request<Body>) -> BoxFutureReq<Vec<u8>> {
        Box::pin(async move {
            let uri = req.uri().clone();
            let method = req.method().clone();
            let version = req.version();
            let headers = req.headers().clone();
            let future_req = body::to_bytes(req.into_body()).await
                .map_err(DBProstError::HyperError)
                .map(move |body|
                    ServiceRequest { uri, method, version, headers, input: body.to_vec() }
                );
            future_req
        })
    }

    /// Turn a byte-array service request into a hyper request
    pub fn to_hyper_raw(&self) -> Request<Body> {
        let mut request = Request::builder()
            .method(self.method.clone())
            .uri(self.uri.clone())
            .body(Body::from(self.input.clone())).unwrap();

        let header_mut = request.headers_mut();
        header_mut.clone_from(&self.headers);
        let lenth_value = header::HeaderValue::from(self.input.len() as u64);
        header_mut.insert(header::CONTENT_LENGTH, lenth_value);
        request
    }

    /// Turn a byte-array service request into a `AfterBodyError`-wrapped version of the given error
    pub fn body_err(&self, err: DBProstError) -> DBProstError {
        DBProstError::AfterBodyError {
            body: self.input.clone(),
            method: Some(self.method.clone()),
            version: self.version,
            headers: self.headers.clone(),
            status: None,
            err: Box::new(err),
        }
    }

    /// Serialize the byte-array service request into a protobuf service request
    pub fn to_proto<T: Message + Default + 'static>(&self) -> Result<ServiceRequest<T>, DBProstError> {
        match T::decode(self.input.as_ref()) {
            Ok(v) => Ok(self.clone_with_input(v)),
            Err(err) => Err(self.body_err(DBProstError::ProstDecodeError(err)))
        }
    }
}

impl<T: Message + Default + 'static> ServiceRequest<T> {
    /// Turn a protobuf service request into a byte-array service request
    pub fn to_proto_raw(&self) -> Result<ServiceRequest<Vec<u8>>, DBProstError> {
        let mut body = Vec::new();
        if let Err(err) = self.input.encode(&mut body) {
            Err(DBProstError::ProstEncodeError(err))
        } else {
            Ok(self.clone_with_input(body))
        }
    }

    /// Turn a hyper request into a protobuf service request
    pub fn from_hyper_proto(req: Request<Body>) -> BoxFutureReq<T> {
        Box::pin(async move {
            ServiceRequest::from_hyper_raw(req).await.and_then(|v| v.to_proto())
        })
    }

    /// Turn a protobuf service request into a hyper request
    pub fn to_hyper_proto(&self) -> Result<Request<Body>, DBProstError> {
        self.to_proto_raw().map(|v| v.to_hyper_raw())
    }
}
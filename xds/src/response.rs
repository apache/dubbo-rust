use hyper::{body, Body, Response, Version, header, StatusCode};
use prost::{Message};
use crate::util::*;
use crate::error::*;


/// A response with HTTP info and a serialized output object
#[derive(Debug)]
pub struct ServiceResponse<T> {
    /// The HTTP version
    pub version: Version,
    /// The set of headers
    ///
    /// Should always at least have `Content-Type`. Servers will override `Content-Length` on serialization.
    pub headers: header::HeaderMap,
    /// The status code
    pub status: StatusCode,
    /// The serialized output object
    pub output: T,
}

impl<T> ServiceResponse<T> {
    /// Create new service request with the given input object
    ///
    /// This automatically sets the `Content-Type` header as `application/protobuf`.
    pub fn new(output: T) -> ServiceResponse<T> {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "application/protobuf".parse().unwrap());

        ServiceResponse {
            version: Version::default(),
            headers,
            status: StatusCode::OK,
            output,
        }
    }

    /// Copy this response with a different output value
    pub fn clone_with_output<U>(&self, output: U) -> ServiceResponse<U> {
        ServiceResponse { version: self.version, headers: self.headers.clone(), status: self.status, output }
    }
}

impl<T: Message + Default + 'static> From<T> for ServiceResponse<T> {
    fn from(v: T) -> ServiceResponse<T> { ServiceResponse::new(v) }
}

impl ServiceResponse<Vec<u8>> {
    /// Turn a hyper response to a boxed future of a byte-array service response
    pub fn from_hyper_raw(resp: Response<Body>) -> BoxFutureResp<Vec<u8>> {
        Box::pin(async move {
            let version = resp.version();
            let headers = resp.headers().clone();
            let status = resp.status().clone();
            let future_resp = body::to_bytes(resp.into_body()).await
                .map_err(DBProstError::HyperError)
                .map(move |body|
                    ServiceResponse { version, headers, status, output: body.to_vec() }
                );
            future_resp
        })
    }

    /// Turn a byte-array service response into a hyper response
    pub fn to_hyper_raw(&self) -> Response<Body> {
        let mut resp = Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(self.output.clone())).unwrap();

        let header_mut = resp.headers_mut();
        header_mut.clone_from(&self.headers);
        let lenth_value = header::HeaderValue::from(self.output.len() as u64);
        header_mut.insert(header::CONTENT_LENGTH, lenth_value);
        resp
    }

    /// Turn a byte-array service response into a `AfterBodyError`-wrapped version of the given error
    pub fn body_err(&self, err: DBProstError) -> DBProstError {
        DBProstError::AfterBodyError {
            body: self.output.clone(),
            method: None,
            version: self.version,
            headers: self.headers.clone(),
            status: Some(self.status),
            err: Box::new(err),
        }
    }


    /// Serialize the byte-array service response into a protobuf service response
    pub fn to_proto<T: Message + Default + 'static>(&self) -> Result<ServiceResponse<T>, DBProstError> {
        if self.status.is_success() {
            match T::decode(self.output.as_ref()) {
                Ok(v) => Ok(self.clone_with_output(v)),
                Err(err) => Err(self.body_err(DBProstError::ProstDecodeError(err)))
            }
        } else {
            match DBError::from_json_bytes(self.status, &self.output) {
                Ok(err) => Err(self.body_err(DBProstError::DBWrapError(err))),
                Err(err) => Err(self.body_err(DBProstError::JsonDecodeError(err)))
            }
        }
    }
}

impl<T: Message + Default + 'static> ServiceResponse<T> {
    /// Turn a protobuf service response into a byte-array service response
    pub fn to_proto_raw(&self) -> Result<ServiceResponse<Vec<u8>>, DBProstError> {
        let mut body = Vec::new();
        if let Err(err) = self.output.encode(&mut body) {
            Err(DBProstError::ProstEncodeError(err))
        } else {
            Ok(self.clone_with_output(body))
        }
    }


    /// Turn a hyper response into a protobuf service response
    pub async fn from_hyper_proto(resp: Response<Body>) -> Result<ServiceResponse<T>, DBProstError> {
        // Box::pin(async move {
        ServiceResponse::from_hyper_raw(resp).await.and_then(|v| v.to_proto())
        // })
    }


    /// Turn a protobuf service response into a hyper response
    pub fn to_hyper_proto(&self) -> Result<Response<Body>, DBProstError> {
        self.to_proto_raw().map(|v| v.to_hyper_raw())
    }
}


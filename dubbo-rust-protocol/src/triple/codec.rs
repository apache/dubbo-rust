use std::{convert::Infallible, marker::PhantomData};

use http::HeaderMap;
// triple codec
use http::request::Request as HttpRequest;
use http::response::Response as HttpResponse;
use hyper::body::HttpBody;
use hyper::Body;

pub struct TripleCodec;

impl TripleCodec {
    // decode from http to triple request
    pub async fn decode_from_http(req: HttpRequest<Body>) -> Result<REQ, ()> {
        let mut req = Box::pin(req);
        match req.data().await {
            Some(Ok(b)) => {
                let msg: Result<REQ, _> = prost::Message::decode(&b[5..]);
                if let Ok(msg) = msg {
                    return Ok(msg);
                }
            }
            _ => {}
        }

        return Err(());
    }

    // encode from triple response to http
    pub async fn encode_to_http(req: super::service::) -> Result<HttpResponse<Body>, ()> {
        let mut buffer = Vec::<u8>::new();
        if let Ok(_) = req.encode(&mut buffer) {
            buffer.insert(0, buffer.len() as u8);
            for _ in 0..4 {
                buffer.insert(0, 0u8);
            }
        }

        let (mut trailer, inner) = Body::channel();

        let mut headers = HeaderMap::new();
        headers.insert("grpc-status", 0i32.into());

        trailer.send_data(buffer.into()).await.unwrap();
        trailer.send_trailers(headers).await.unwrap();

        let resp = HttpResponse::builder().status(200).body(inner).unwrap();

        return Ok(resp);
    }
}

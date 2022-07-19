use std::str::FromStr;

use super::echo_server::{HelloReply, HelloRequest};
use bytes::Buf;

use triple::client::TripleClient;
use triple::codec::serde_codec::SerdeCodec;
use triple::invocation::*;
use triple::server::Streaming;

pub struct EchoClient {
    inner: TripleClient,
    uri: String,
}

impl EchoClient {
    pub fn new() -> Self {
        Self {
            inner: TripleClient::new(),
            uri: "".to_string(),
        }
    }

    pub fn with_uri(mut self, uri: String) -> Self {
        self.uri = uri;
        self.inner = self
            .inner
            .with_authority(http::uri::Authority::from_str(&self.uri).unwrap());
        self
    }

    // pub async fn connect(&self, url: &str) {
    //     self.inner.request(req)
    // }

    pub async fn bidirectional_streaming_echo(
        mut self,
        req: impl IntoStreamingRequest<Message = HelloRequest>,
    ) -> Result<Response<Streaming<HelloReply>>, tonic::Status> {
        let codec = SerdeCodec::<HelloRequest, HelloReply>::default();
        self.inner
            .bidi_streaming(
                req,
                codec,
                http::uri::PathAndQuery::from_static("/bidi_stream"),
            )
            .await
        // Stream trait to Body
        // let mut codec = SerdeCodec::<HelloRequest, HelloReply>::default();
        // let stream = req.into_streaming_request();
        // let en = encode(codec.encoder(), stream.into_inner().map(Ok));
        // let body = hyper::Body::wrap_stream(en);

        // let req = http::Request::builder()
        //     .version(Version::HTTP_2)
        //     .uri(self.uri.clone() + "/bidi_stream")
        //     .method("POST")
        //     .body(body)
        //     .unwrap();

        // let response = self.inner.request(req).await;

        // match response {
        //     Ok(v) => {
        //         println!("response: {:?}", v);
        //         // println!("grpc status: {:?}", v)
        //         let mut resp = v.map(|body| Streaming::new(body, codec.decoder()));
        //         // TODO: rpc response to http response
        //         let trailers_only_status = tonic::Status::from_header_map(resp.headers_mut());
        //         println!("trailer only status: {:?}", trailers_only_status);

        //         let (parts, mut body) = resp.into_parts();
        //         let trailer = body.trailer().await.unwrap();
        //         println!("trailer: {:?}", trailer);

        //         // if let Some(trailer) = trailer.take() {
        //         //     println!("trailer: {:?}", trailer);
        //         // }
        //         return Ok(Response::new(body));
        //     }
        //     Err(err) => {
        //         println!("error: {}", err);
        //         return Err(tonic::Status::new(tonic::Code::Internal, err.to_string()));
        //     }
        // }
    }

    pub async fn say_hello(
        &self,
        req: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, tonic::Status> {
        let (_parts, body) = req.into_parts();
        let v = serde_json::to_vec(&body).unwrap();
        let req = hyper::Request::builder()
            .uri("http://".to_owned() + &self.uri.clone() + "/hello")
            .method("POST")
            .body(hyper::Body::from(v))
            .unwrap();

        println!("request: {:?}", req);
        let response = hyper::Client::builder().build_http().request(req).await;

        match response {
            Ok(v) => {
                println!("{:?}", v);
                let (_parts, body) = v.into_parts();
                let req_body = hyper::body::to_bytes(body).await.unwrap();
                let v = req_body.chunk();
                // let codec = SerdeCodec::<HelloReply, HelloRequest>::default();
                let data: HelloReply = match serde_json::from_slice(v) {
                    Ok(data) => data,
                    Err(err) => {
                        return Err(tonic::Status::new(tonic::Code::Internal, err.to_string()))
                    }
                };
                Ok(Response::new(data))
            }
            Err(err) => {
                println!("{}", err);
                return Err(tonic::Status::new(tonic::Code::Internal, err.to_string()));
            }
        }
    }
}

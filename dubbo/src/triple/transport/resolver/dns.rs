use std::future::Future;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::pin::Pin;
use std::task::Poll;
use tokio::task::JoinHandle;

use tower_service::Service;

#[derive(Clone, Default)]
pub struct DnsResolver {}

impl Service<String> for DnsResolver {
    type Response = Vec<SocketAddr>;

    type Error = std::io::Error;

    type Future = DnsFuture;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, name: String) -> Self::Future {
        let block = tokio::task::spawn_blocking(move || {
            (name, 0)
                .to_socket_addrs()
                .map(|res| res.as_slice().to_vec())
        });

        DnsFuture { inner: block }
    }
}

pub struct DnsFuture {
    inner: JoinHandle<Result<Vec<SocketAddr>, std::io::Error>>,
}

impl Future for DnsFuture {
    type Output = Result<Vec<SocketAddr>, std::io::Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|res| match res {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(err)) => Err(err),
            Err(join_err) => {
                if join_err.is_cancelled() {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Interrupted,
                        join_err,
                    ))
                } else {
                    panic!("dnsfuture poll failed: {:?}", join_err)
                }
            }
        })
    }
}

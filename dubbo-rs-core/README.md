# dubbo-rs-core

`dubbo-rs-core` is a small, byte-oriented facade for embedding Dubbo Rust in
other runtimes.

The crate is intentionally lower level than the generated Rust client APIs. It
keeps the host language in control of service descriptors, protobuf
serialization, and public API shape, while Rust owns Dubbo transport and
governance behavior such as Triple calls, registry discovery, load balancing,
cluster strategy, routing, and timeouts.

This boundary is useful for packages such as `dubbo-js`, where JavaScript should
keep the user-facing client API while a N-API addon reuses the Rust core.

## Current Surface

- Raw unary Triple requests and responses.
- Static provider endpoints.
- Registry-backed clients behind optional features.
- Request-level and client-default unary timeouts.
- Load balancing: `random`, `round_robin`, and `p2c`.
- Cluster strategy: `failfast` and `failover`.
- Metadata routing for tag, group, and version.
- Stable status code and message access for host-language error mapping.

## Features

- `registry-nacos`: enable Nacos registry support.
- `registry-zookeeper`: enable Zookeeper registry support.
- `registry`: enable all registry backends currently wired into this crate.

## Example

```rust
use bytes::Bytes;
use dubbo_rs_core::{
    RawMetadata, RawTripleClient, RawTripleClientOptions, RawUnaryRequest,
};

# async fn call() -> Result<(), Box<dyn std::error::Error>> {
let mut client = RawTripleClient::from_static_endpoints_with_options(
    ["http://127.0.0.1:50051?interface=example.EchoService"],
    RawTripleClientOptions {
        timeout_ms: Some(3_000),
        load_balance: Some("round_robin".to_string()),
        cluster: Some("failfast".to_string()),
    },
)?;

let response = client
    .unary(RawUnaryRequest {
        service: "example.EchoService".to_string(),
        method: "Echo".to_string(),
        path: "/example.EchoService/Echo".to_string(),
        metadata: RawMetadata::new().insert("tri-service-group", "default"),
        body: Bytes::from_static(b"\x0a\x05hello"),
        timeout_ms: None,
    })
    .await?;

let _bytes = response.body;
# Ok(())
# }
```

The request and response bodies are protobuf bytes. Host runtimes should encode
and decode messages using their own protobuf toolchain.

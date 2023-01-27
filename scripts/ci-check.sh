#!/bin/bash

cargo fmt --all -- --check
# use stable channel
cargo check
target/debug/greeter-server && target/debug/greeter-client && sleep 3s ;

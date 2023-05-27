#!/bin/bash
#alias rust-musl-builder='docker run --rm -it -v "$(pwd)":/home/rust/src -v /Users/dqh/.cargo/git:/opt/rust/cargo/git -v /Users/dqh/.cargo/registry:/opt/rust/cargo/registry -v /Users/dqh/.cargo/config:/opt/rust/cargo/config  ghcr.io/calm04061/rust_musl_builder:openssl'
#rust-musl-builder  cargo build --release

cargo build --target aarch64-unknown-linux-musl
#alias rust-musl-builder='docker run --rm -it -v "$(pwd)":/home/rust/src  -v "/Users/dqh/.cargo/registry":/root/.cargo/registry   calm04061/rust-musl-cross:aarch64-musl'

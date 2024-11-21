## fctools guest agents

This repository contains two guest agent servers that serve over virtio-vsock, the first using plain HTTP/1.1 and the second using gRPC with HTTP/2.

These guest agents are used in the test suite for `fctools` to ensure the functionality of the HTTP-over-vsock and gRPC-over-vsock extensions.

The HTTP guest agent contains a single POST `/ping` route that accepts JSON and returns JSON, while the gRPC guest agent contains 4 ping-like methods:
unary, client streaming, server streaming and duplex streaming.

Prebuilt binaries of both guest agents are available on the releases page and in the `testdata` package of `fctools`. Compiling them from source is
easy as well, run `cargo build -r` in the workspace after ensuring that a stable Rust toolchain is installed alongside `protoc` and the
`x86_64-unknown-linux-musl` target.

# Common Protobuf Messages

We're using protobuf as the common messaging format for between our embedded devices and host machine. This crate contains the common protobuf messages that are used by both ground station and the boards.

# Prost
The Rust prost library is a fast, efficient implementation of Google Protocol Buffers (protobuf) for Rust.
- It provides a way to define structured data in `.proto` files and automatically generates Rust types from them.
- The generated Rust code is strongly typed and memory-safe.

## Building Protobuf Messages for Rust
Run `cargo make rust-bindings`

## Building Protobuf Messages for Python
Run `cargo make python-bindings`. Make sure you have protobuf installed with python: `pip install protobuf`.
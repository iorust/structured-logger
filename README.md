# structured-logger

![License](https://img.shields.io/crates/l/structured-logger.svg)
[![Crates.io](https://img.shields.io/crates/d/structured-logger.svg)](https://crates.io/crates/structured-logger)
[![Codecov](https://codecov.io/gh/iorust/structured-logger/branch/main/graph/badge.svg)](https://codecov.io/gh/iorust/structured-logger)
[![CI](https://github.com/iorust/structured-logger/actions/workflows/ci.yml/badge.svg)](https://github.com/iorust/structured-logger/actions/workflows/ci.yml)
[![Docs.rs](https://img.shields.io/docsrs/structured-logger?label=docs.rs)](https://docs.rs/structured-logger)
[![Latest Version](https://img.shields.io/crates/v/structured-logger.svg)](https://crates.io/crates/structured-logger)

A logging implementation for the log crate that logs structured values either synchronous or asynchronous, in JSON, CBOR, or any other format, to a file, stderr, stdout, or any other destination.

It is inspired by [std-logger](https://github.com/Thomasdezeeuw/std-logger).

## Usage

See examples and the [API documentation] for more.

## Example

Simple example:
```rust
use serde::Serialize;
use structured_logger::{async_json::new_writer, unix_ms, Builder};

#[tokio::main]
async fn main() {
    // Initialize the logger.
    Builder::with_level("info")
        .with_target_writer("*", new_writer(tokio::io::stdout()))
        .init();

    let kv = ContextLog {
        uid: "user123".to_string(),
        action: "upate_book".to_string(),
    };

    log::info!("hello world");
    // This log will be written to stdout:
    // {"level":"INFO","message":"hello world","target":"simple","timestamp":1679745592127}

    log::info!(target: "api",
        method = "GET",
        path = "/hello",
        status = 200_u16,
        start = unix_ms(),
        elapsed = 10_u64,
        kv:serde = kv;
        "",
    );
    // This log will be written to stdout:
    // {"elapsed":10,"kv":{"uid":"user123","action":"upate_book"},"level":"INFO","message":"","method":"GET","path":"/hello","start":1679745592127,"status":200,"target":"api","timestamp":1679745592127}
}

#[derive(Serialize)]
struct ContextLog {
    uid: String,
    action: String,
}
```

* Log panics example: https://github.com/iorust/structured-logger/blob/main/examples/panic_log.rs
* Async log example: https://github.com/iorust/structured-logger/blob/main/examples/async_log.rs
* Custom writers example: https://github.com/iorust/structured-logger/blob/main/examples/custom.rs

[API documentation]: https://docs.rs/structured-logger

## License
Copyright © 2023-present [IO Rust](https://github.com/iorust).

`iorust/structured-logger` is licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE">MIT license</a> at your option.

# structured-logger

![License](https://img.shields.io/crates/l/structured-logger.svg)
[![Crates.io](https://img.shields.io/crates/d/structured-logger.svg)](https://crates.io/crates/structured-logger)
[![CI](https://github.com/iorust/structured-logger/actions/workflows/ci.yml/badge.svg)](https://github.com/iorust/structured-logger/actions/workflows/ci.yml)
[![Docs.rs](https://img.shields.io/docsrs/structured-logger?label=docs.rs)](https://docs.rs/structured-logger)
[![Latest Version](https://img.shields.io/crates/v/structured-logger.svg)](https://crates.io/crates/structured-logger)

A logging implementation for the log crate that logs structured values as JSON (CBOR, or any other) into a file, stderr, stdout, or any other.
Inspired by [std-logger](https://github.com/Thomasdezeeuw/std-logger).

## Usage

See the [API documentation] for more.

## Example

Log panics example: https://github.com/iorust/structured-logger/blob/main/examples/panic_log.rs

Simple example:
```rust
use serde::Serialize;
use std::{fs::File, io::stdout};
use structured_logger::{json::new_json_writer, unix_ms, Logger};

fn main() {
    // Initialize the logger.
    let log_file = File::options()
        .create(true)
        .append(true)
        .open("app.log")
        .unwrap();

    Logger::new()
        // set a specific writer (format to JSON, write to stdout) for target "api".
        .with_target_writer("api", new_json_writer(stdout()))
        // set a specific writer (format to JSON, write to app.log file) for target "file".
        .with_target_writer("file", new_json_writer(log_file))
        .init();

    let kv = ContextLog {
        uid: "user123".to_string(),
        action: "upate_book".to_string(),
    };

    log::info!("hello world");
    // This log will be written to stderr (default writer):
    // {"level":"INFO","message":"hello world","target":"simple","timestamp":1679745592127}

    log::info!(target: "api",
        method = "GET",
        path = "/hello",
        status = 200_u16,
        start = unix_ms(),
        elapsed = 10_u64,
        kv = log::as_serde!(kv);
        "",
    );
    // This log will be written to stdout:
    // {"elapsed":10,"kv":{"uid":"user123","action":"upate_book"},"level":"INFO","message":"","method":"GET","path":"/hello","start":1679745592127,"status":200,"target":"api","timestamp":1679745592127}

    log::info!(target: "file",
        method = "GET",
        path = "/hello",
        status = 200_u16,
        start = unix_ms(),
        elapsed = 10_u64,
        kv = log::as_serde!(kv);
        "",
    );
    // This log will be written to file "app.log":
    // {"elapsed":10,"kv":{"uid":"user123","action":"upate_book"},"level":"INFO","message":"","method":"GET","path":"/hello","start":1679745592127,"status":200,"target":"file","timestamp":1679745592127}
}

#[derive(Serialize)]
struct ContextLog {
    uid: String,
    action: String,
}
```

[API documentation]: https://docs.rs/structured-logger

## License
Copyright © 2023-present [IO Rust](https://github.com/iorust).

`iorust/structured-logger` is licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE">MIT license</a> at your option.

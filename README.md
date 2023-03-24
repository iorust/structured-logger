# structured-logger
[![CI](https://github.com/iorust/structured-logger/actions/workflows/ci.yml/badge.svg)](https://github.com/iorust/structured-logger/actions/workflows/ci.yml)
[![License](http://img.shields.io/badge/license-mit-blue.svg?style=flat-square)](https://raw.githubusercontent.com/iorust/structured-logger/main/LICENSE)

A logging implementation for the log crate that logs structured values as JSON (CBOR, or any other) into a file, stderr, stdout, or any other.

## Usage

See the [API documentation] for more.

## Example

```rust
use serde::Serialize;
use std::io::stdout;
use structured_logger::{json::new_json_writer, unix_ms, Logger};

fn main() {
    // Initialize the logger.
    Logger::new()
        // set a specific writer (format to JSON, write to stdout) for target "request".
        .with_target_writer("request", new_json_writer(stdout()))
        .init();

    let kv = ContextLog {
        uid: "user123".to_string(),
        action: "upate_book".to_string(),
    };

    log::info!("hello world");
    // {"level":"INFO","message":"hello world","target":"simple","timestamp":1679655670735}

    // mock request data
    log::info!(target: "request",
        method = "GET",
        path = "/hello",
        status = 200 as u16,
        start = unix_ms(),
        elapsed = 10 as u64,
        kv = log::as_serde!(kv);
        "",
    );
    // {"elapsed":10,"kv":{"uid":"user123","action":"upate_book"},"level":"INFO","message":"","method":"GET","path":"/hello","start":1679655670735,"status":200,"target":"request","timestamp":1679655670735}
}

#[derive(Serialize)]
struct ContextLog {
    uid: String,
    action: String,
}
```

[API documentation]: https://docs.rs/structured-logger

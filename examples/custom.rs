use serde::Serialize;
use std::{fs::File, io::stdout};
use structured_logger::{json::new_writer, unix_ms, Builder};

fn main() {
    // Initialize the logger.
    // Optional: create a file to write logs to.
    let log_file = File::options()
        .create(true)
        .append(true)
        .open("app.log")
        .unwrap();

    // or Builder::with_level("debug")
    Builder::new()
        // Optional: set a specific writer (format to JSON, write to stdout) for target starts with "api"..
        .with_target_writer("api*", new_writer(stdout()))
        // Optional: set a specific writer (format to JSON, write to app.log file) for target "file" and "db".
        .with_target_writer("file,db", new_writer(log_file))
        .with_msg_field() // use "msg" field for log message instead of "message"
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
        kv:serde = kv;
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
        kv:serde = kv;
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

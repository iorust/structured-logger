use log;
use serde::Serialize;
use std::{io, time::SystemTime, time::UNIX_EPOCH};
use structured_logger::{new_json_writer, Logger};

fn main() {
    // Initialize the logger.
    Logger::new()
        // set a specific writer (format to JSON, write to stdout) for target "request".
        .with_target_writer("request", new_json_writer(io::stdout()))
        .init();

    let kv = ContextLog {
        uid: "user123".to_string(),
        action: "upate_book".to_string(),
    };

    log::info!("hello world");
    // {"target":"simple","message":"hello world","level":"INFO"}

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
    // {"method":"GET","target":"request","message":"","path":"/hello","status":200,"level":"INFO","start":1679647263247,"kv":{"uid":"user123","action":"upate_book"},"elapsed":10}
}

#[derive(Serialize)]
struct ContextLog {
    uid: String,
    action: String,
}

fn unix_ms() -> u64 {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before Unix epoch");
    ts.as_millis() as u64
}

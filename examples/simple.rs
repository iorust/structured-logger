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

use serde::Serialize;
use structured_logger::{async_json::new_writer, unix_ms, Builder};
use tokio::{io, time};

#[tokio::main]
async fn main() {
    // Initialize the logger.
    // Builder::with_level("debug")
    Builder::new()
        // Optional: set a specific async writer (format to JSON, write to stdout) for target "api".
        .with_target_writer("api", new_writer(io::stdout()))
        .init();

    loop {
        log::info!("hello world");
        // This log will be written to stderr (default writer):
        // {"level":"INFO","message":"hello world","target":"async_log","timestamp":1679973279977}

        let kv = ContextLog {
            uid: "user123".to_string(),
            action: "upate_book".to_string(),
        };

        log::info!(target: "api",
            method = "GET",
            path = "/hello",
            status = 200_u16,
            start = unix_ms(),
            elapsed = 10_u64,
            kv:serde = kv;
            "",
        );
        // This log will be written to tokio stdout (async writer):
        // {"elapsed":10,"kv":{"uid":"user123","action":"upate_book"},"level":"INFO","message":"","method":"GET","path":"/hello","start":1679973279977,"status":200,"target":"api","timestamp":1679973279977}

        time::sleep(time::Duration::from_secs(1)).await;
    }
}

#[derive(Serialize)]
struct ContextLog {
    uid: String,
    action: String,
}

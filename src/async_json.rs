// (c) 2023-present, IO Rust. All rights reserved.
// See the file LICENSE for licensing terms.

//! # Async JSON Writer Implementation
//!
//! A [`Writer`] implementation that logs structured values
//! asynchronous as JSON into a file, stderr, stdout, or any other.
//! To create a `Box<dyn Writer>` use [`new_writer`].
//!
//! Example: <https://github.com/iorust/structured-logger/blob/main/examples/async_log.rs>
//!

use std::{io, io::Write, pin::Pin, sync::Arc};
use tokio::{io::AsyncWrite, sync::Mutex};

use crate::{unix_ms, Log, Writer};

/// A Writer implementation that writes logs asynchronous in JSON format.
pub struct AsyncJSONWriter<W: AsyncWrite + Sync + Send + 'static>(Arc<Mutex<Pin<Box<W>>>>);

impl<W: AsyncWrite + Sync + Send + 'static> AsyncJSONWriter<W> {
    /// Creates a new AsyncJSONWriter instance.
    pub fn new(w: W) -> Self {
        Self(Arc::new(Mutex::new(Box::pin(w))))
    }
}

/// Implements Writer trait for AsyncJSONWriter.
impl<W: AsyncWrite + Sync + Send + 'static> Writer for AsyncJSONWriter<W> {
    fn write_log(&self, value: &Log) -> Result<(), io::Error> {
        let mut buf = Vec::with_capacity(256);
        serde_json::to_writer(&mut buf, value).map_err(io::Error::from)?;
        // must write the LINE FEED character.

        buf.write_all(b"\n").map_err(io::Error::from)?;

        let w = self.0.clone();
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;

            let mut w = w.lock().await;
            if let Err(err) = w.as_mut().write_all(&buf).await {
                // should never happen, but if it does, we log it.
                eprintln!(
                    "{{\"level\":\"ERROR\",\"message\":\"failed to write log: {}\",\"target\":\"AsyncJSONWriter\",\"timestamp\":{}}}",
                    err, unix_ms(),
                );
            }
        });
        Ok(())
    }
}

/// Creates a new `Box<dyn Writer>` instance with AsyncJSONWriter for a given std::io::Write instance.
pub fn new_writer<W: AsyncWrite + Sync + Send + 'static>(w: W) -> Box<dyn Writer> {
    Box::new(AsyncJSONWriter::new(w))
}
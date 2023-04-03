// (c) 2023-present, IO Rust. All rights reserved.
// See the file LICENSE for licensing terms.

//! # Sync JSON Writer Implementation
//!
//! A [`Writer`] implementation that logs structured values
//! synchronous in JSON format to a file, stderr, stdout, or any other destination.
//! To create a `Box<dyn Writer>` use the [`new_writer`] function.
//!
//! Example: <https://github.com/iorust/structured-logger/blob/main/examples/simple.rs>
//!

use parking_lot::Mutex;
use std::{cell::RefCell, collections::BTreeMap, io, io::Write};

use crate::{log_failure, Key, Value, Writer};
/// A Writer implementation that writes logs in JSON format.
pub struct JSONWriter<W: Write + Sync + Send + 'static>(Mutex<RefCell<Box<W>>>);

impl<W: Write + Sync + Send + 'static> JSONWriter<W> {
    /// Creates a new JSONWriter instance.
    pub fn new(w: W) -> Self {
        Self(Mutex::new(RefCell::new(Box::new(w))))
    }
}

/// Implements Writer trait for JSONWriter.
impl<W: Write + Sync + Send + 'static> Writer for JSONWriter<W> {
    fn write_log(&self, value: &BTreeMap<Key, Value>) -> Result<(), io::Error> {
        let mut buf = Vec::with_capacity(256);
        serde_json::to_writer(&mut buf, value).map_err(io::Error::from)?;
        // must write the LINE FEED character.
        buf.write_all(b"\n").map_err(io::Error::from)?;

        let w = self.0.lock();
        if let Ok(mut w) = w.try_borrow_mut() {
            w.as_mut().write_all(&buf).map_err(io::Error::from)?;
        } else {
            // should never happen, but if it does, we log it.
            log_failure("JSONWriter failed to write log: writer already borrowed");
        }
        Ok(())
    }
}

/// Creates a new `Box<dyn Writer>` instance with the JSONWriter for a given std::io::Write instance.
pub fn new_writer<W: Write + Sync + Send + 'static>(w: W) -> Box<dyn Writer> {
    Box::new(JSONWriter::new(w))
}

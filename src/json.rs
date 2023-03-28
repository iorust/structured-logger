// (c) 2023-present, IO Rust. All rights reserved.
// See the file LICENSE for licensing terms.

//! # Sync JSON Writer Implementation
//!
//! A [`Writer`] implementation that logs structured values
//! synchronous as JSON into a file, stderr, stdout, or any other.
//! To create a `Box<dyn Writer>` use [`new_writer`].
//!
//! Example: <https://github.com/iorust/structured-logger/blob/main/examples/simple.rs>
//!

use parking_lot::Mutex;
use std::{cell::RefCell, io};

use crate::{Log, Writer};
/// A Writer implementation that writes logs in JSON format.
pub struct JSONWriter<W: io::Write + Sync + Send + 'static>(Mutex<RefCell<Box<W>>>);

impl<W: io::Write + Sync + Send + 'static> JSONWriter<W> {
    /// Creates a new JSONWriter instance.
    pub fn new(w: W) -> Self {
        Self(Mutex::new(RefCell::new(Box::new(w))))
    }
}

/// Implements Writer trait for JSONWriter.
impl<W: io::Write + Sync + Send + 'static> Writer for JSONWriter<W> {
    fn write_log(&self, value: &Log) -> Result<(), io::Error> {
        let w = self.0.lock();
        serde_json::to_writer(w.borrow_mut().as_mut(), value).map_err(io::Error::from)?;
        // must write the LINE FEED character.
        w.borrow_mut()
            .as_mut()
            .write_all(b"\n")
            .map_err(io::Error::from)?;
        Ok(())
    }
}

/// Creates a new `Box<dyn Writer>` instance with JSONWriter for a given std::io::Write instance.
pub fn new_writer<W: io::Write + Sync + Send + 'static>(w: W) -> Box<dyn Writer> {
    Box::new(JSONWriter::new(w))
}

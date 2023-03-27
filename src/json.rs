// (c) 2022-present, IO Rust. All rights reserved.
// See the file LICENSE for licensing terms.

use std::{cell::RefCell, io};

use crate::{Log, Writer};

use parking_lot::Mutex;

// TODO: async writer.
/// A Writer implementation that writes logs in JSON format.
pub struct JSONWriter<W: io::Write + Sync + Send>(Mutex<RefCell<Box<W>>>);

/// Implements Writer trait for JSONWriter.
impl<W: io::Write + Sync + Send> Writer for JSONWriter<W> {
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

/// Creates a new JSONWriter instance with a given std::io::Write instance.
pub fn new_json_writer<W: io::Write + Sync + Send + 'static>(w: W) -> Box<dyn Writer> {
    Box::new(JSONWriter(Mutex::new(RefCell::new(Box::new(w)))))
}

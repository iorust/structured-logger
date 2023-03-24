use std::{cell::RefCell, io};

use crate::{Log, Writer};

/// JSONWriter is a Writer implementation that writes logs in JSON format.
pub struct JSONWriter<W: io::Write + Sync + Send>(RefCell<Box<W>>);

/// implementate Writer for JSONWriter.
impl<W: io::Write + Sync + Send> Writer for JSONWriter<W> {
    fn write_log(&self, value: &Log) -> Result<(), io::Error> {
        serde_json::to_writer(self.0.borrow_mut().as_mut(), value).map_err(io::Error::from)?;
        // must write the LINE FEED character.
        self.0
            .borrow_mut()
            .as_mut()
            .write_all(b"\n")
            .map_err(io::Error::from)?;
        Ok(())
    }
}

/// new_json_writer creates a new JSONWriter with a given io::Write instance.
pub fn new_json_writer<W: io::Write + Sync + Send + 'static>(w: W) -> Box<dyn Writer> {
    Box::new(JSONWriter(RefCell::new(Box::new(w))))
}

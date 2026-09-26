//! Ready-made JSON Lines readers and writers for common destinations.
//!
//! These thin constructors keep the generic [`codec`](crate::codec) machinery out of call sites:
//! the names say what is read or written and to what medium. The format is JSON Lines; for another
//! format implement [`RecordWriter`](crate::codec::RecordWriter)/[`RecordReader`](crate::codec::RecordReader)
//! directly.

use crate::codec::{JsonRecordReader, JsonRecordWriter};
use std::cell::RefCell;
use std::fs::File;
use std::io::{BufReader, Cursor, Write};
use std::path::Path;
use std::rc::Rc;

/// In-memory sink shared with a writer, so the bytes written can be read back later.
#[derive(Clone, Default)]
pub struct SharedBytes(Rc<RefCell<Vec<u8>>>);

impl SharedBytes {
    pub fn new() -> Self {
        Self::default()
    }

    /// Copies everything written so far.
    pub fn bytes(&self) -> Vec<u8> {
        self.0.borrow().clone()
    }
}

impl Write for SharedBytes {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// JSON Lines writer over the file at `path`, created or truncated.
pub fn json_journal_file_writer(path: impl AsRef<Path>) -> Result<JsonRecordWriter<File>, String> {
    let file = File::create(path).map_err(|error| error.to_string())?;
    Ok(JsonRecordWriter::new(file))
}

/// JSON Lines reader over the file at `path`.
pub fn json_journal_file_reader(
    path: impl AsRef<Path>,
) -> Result<JsonRecordReader<BufReader<File>>, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    Ok(JsonRecordReader::new(BufReader::new(file)))
}

/// JSON Lines writer filling `sink`; read the result back with [`SharedBytes::bytes`].
pub fn json_journal_memory_writer(sink: &SharedBytes) -> JsonRecordWriter<SharedBytes> {
    JsonRecordWriter::new(sink.clone())
}

/// JSON Lines reader over in-memory `bytes`.
pub fn json_journal_memory_reader(bytes: Vec<u8>) -> JsonRecordReader<Cursor<Vec<u8>>> {
    JsonRecordReader::new(Cursor::new(bytes))
}

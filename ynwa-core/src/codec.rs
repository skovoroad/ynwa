//! Byte-level recording formats.
//!
//! The in-memory [`Record`] model is format-independent: this module only converts it to and
//! from a byte stream. The default codec is JSON Lines (one JSON object per line), chosen for
//! readability; a binary codec can be plugged in later without touching the model or the
//! recording points.

use crate::game::GameState;
use crate::journal::{JournalEntry, JournalEvent, JournalSink};
use crate::record::{Record, RecordFooter, RecordHeader};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};

/// Streaming writer of a recording; the format is implementation-defined.
pub trait RecordWriter {
    fn write_header(&mut self, header: &RecordHeader) -> Result<(), String>;
    fn write_event(&mut self, entry: &JournalEntry) -> Result<(), String>;

    /// Writes the trailer carrying the final state of the playthrough.
    fn finish(&mut self, footer: &RecordFooter) -> Result<(), String>;
}

/// Reads a byte stream back into a [`Record`].
pub trait RecordReader {
    fn read(&mut self) -> Result<Record, String>;
}

/// One line of a JSON Lines recording.
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum RecordLine {
    // Boxed because a header carries the full initial `GameState` and would otherwise dwarf
    // the other variants.
    Header(Box<RecordHeader>),
    Event(JournalEntry),
    Footer(RecordFooter),
}

/// Writes [`RecordLine`]s, flushing after each one, so a crash leaves a valid prefix on disk.
pub struct JsonRecordWriter<W: Write> {
    sink: W,
}

impl<W: Write> JsonRecordWriter<W> {
    pub fn new(sink: W) -> Self {
        Self { sink }
    }

    fn write_line(&mut self, line: &RecordLine) -> Result<(), String> {
        let mut encoded = serde_json::to_string(line).map_err(|error| error.to_string())?;
        encoded.push('\n');
        self.sink
            .write_all(encoded.as_bytes())
            .map_err(|error| error.to_string())?;
        self.sink.flush().map_err(|error| error.to_string())
    }
}

impl<W: Write> RecordWriter for JsonRecordWriter<W> {
    fn write_header(&mut self, header: &RecordHeader) -> Result<(), String> {
        self.write_line(&RecordLine::Header(Box::new(header.clone())))
    }

    fn write_event(&mut self, entry: &JournalEntry) -> Result<(), String> {
        self.write_line(&RecordLine::Event(entry.clone()))
    }

    fn finish(&mut self, footer: &RecordFooter) -> Result<(), String> {
        self.write_line(&RecordLine::Footer(footer.clone()))
    }
}

/// Reads JSON Lines back into a [`Record`]. A line cut off mid-write (no trailing newline) is
/// ignored, so a crashed recording still yields its valid prefix.
pub struct JsonRecordReader<R: BufRead> {
    source: R,
}

impl<R: BufRead> JsonRecordReader<R> {
    pub fn new(source: R) -> Self {
        Self { source }
    }
}

impl<R: BufRead> RecordReader for JsonRecordReader<R> {
    fn read(&mut self) -> Result<Record, String> {
        let mut header = None;
        let mut journal = Vec::new();
        let mut footer = RecordFooter::default();
        let mut line = Vec::new();

        loop {
            line.clear();
            let bytes_read = self
                .source
                .read_until(b'\n', &mut line)
                .map_err(|error| error.to_string())?;
            if bytes_read == 0 {
                break;
            }
            if line.last() != Some(&b'\n') {
                break;
            }
            let text = std::str::from_utf8(&line).map_err(|error| error.to_string())?;
            match serde_json::from_str::<RecordLine>(text.trim())
                .map_err(|error| error.to_string())?
            {
                RecordLine::Header(value) => {
                    if header.is_some() {
                        return Err("recording contains more than one header".to_string());
                    }
                    header = Some(*value);
                }
                RecordLine::Event(entry) => journal.push(entry),
                RecordLine::Footer(value) => footer = value,
            }
        }

        let header = header.ok_or_else(|| "recording contains no header".to_string())?;
        Ok(Record {
            header,
            footer,
            journal,
        })
    }
}

/// Sink that writes every event straight into a [`RecordWriter`].
///
/// [`JournalSink::push`] cannot report write errors, so the first one is kept and returned by
/// [`JournalSink::finish`].
pub struct FileJournalRecorder {
    writer: Box<dyn RecordWriter>,
    first_error: Option<String>,
}

impl FileJournalRecorder {
    /// Creates a recorder over `writer` and writes the header immediately.
    pub fn new(writer: Box<dyn RecordWriter>, header: &RecordHeader) -> Self {
        let mut recorder = Self {
            writer,
            first_error: None,
        };
        if let Err(error) = recorder.writer.write_header(header) {
            recorder.first_error = Some(error);
        }
        recorder
    }
}

impl JournalSink for FileJournalRecorder {
    fn push(&mut self, timestamp: f32, event: JournalEvent) {
        let entry = JournalEntry { timestamp, event };
        if let Err(error) = self.writer.write_event(&entry) {
            self.first_error.get_or_insert(error);
        }
    }

    fn finish(mut self: Box<Self>, final_state: &GameState) -> Result<(), String> {
        let footer = RecordFooter::from_state(final_state);
        if let Err(error) = self.writer.finish(&footer) {
            self.first_error.get_or_insert(error);
        }
        match self.first_error.take() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

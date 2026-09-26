//! Tests for the JSON Lines codec and the file journal recorder.

use crate::codec::{FileJournalRecorder, JsonRecordCodec, RecordReader, RecordWriter};
use crate::field::zones::Velocity3D;
use crate::field::Field;
use crate::game::{
    BallDef, Decision, DecisionTarget, GameConfig, GameStage, PlayerDef, RefereeDef,
    REGION_START_POSITION,
};
use crate::journal::{JournalEntry, JournalEvent, JournalSink};
use crate::record::RecordHeader;
use crate::region::GridCell;
use crate::team::Team;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::{BufReader, Cursor, Write};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

const FOOTER_MARKER: &[u8] = b"{\"type\":\"footer\"";

fn test_config() -> GameConfig {
    let field = Field::from_meters(100.0, 60.0, 26, 44);
    let grid_dims = field.grid_dimensions();
    let start_region = grid_dims
        .create_region(GridCell::new(1, 1).unwrap(), GridCell::new(2, 2).unwrap())
        .unwrap();

    GameConfig {
        field,
        players: vec![PlayerDef::new(
            Team::A,
            1,
            "Test Player".to_string(),
            String::new(),
            HashMap::from([(REGION_START_POSITION.to_string(), start_region)]),
        )],
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: crate::game::ScriptingConfig::empty(),
    }
}

fn test_header() -> RecordHeader {
    RecordHeader {
        config: test_config(),
        initial_stage: GameStage::Setup("kick off".to_string()),
        fixed_dt: 0.016,
    }
}

fn sample_entries() -> Vec<JournalEntry> {
    vec![
        JournalEntry {
            timestamp: 0.016,
            event: JournalEvent::DecisionAssigned {
                player_index: 0,
                decision: Decision::Run(DecisionTarget::GridCell(GridCell::new(1, 1).unwrap())),
                reason: Some("attack".to_string()),
            },
        },
        JournalEntry {
            timestamp: 0.032,
            event: JournalEvent::KickOutcome {
                player_index: 0,
                ball_velocity: Velocity3D::from_meters_per_second(1.0, 0.0, -2.0),
            },
        },
        JournalEntry {
            timestamp: 0.048,
            event: JournalEvent::StageChange {
                stage: GameStage::Play,
            },
        },
    ]
}

/// Owned write sink that can be cloned to observe bytes while the recorder still owns a handle.
#[derive(Clone, Default)]
struct SharedBuffer(Rc<RefCell<Vec<u8>>>);

impl SharedBuffer {
    fn contents(&self) -> Vec<u8> {
        self.0.borrow().clone()
    }

    fn text(&self) -> String {
        String::from_utf8(self.contents()).unwrap()
    }
}

impl Write for SharedBuffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn write_recording(header: &RecordHeader, entries: &[JournalEntry], total_steps: u64) -> Vec<u8> {
    let sink = SharedBuffer::default();
    let mut recorder =
        FileJournalRecorder::new(Box::new(JsonRecordCodec::writer(sink.clone())), header);
    for entry in entries {
        recorder.push(entry.timestamp, entry.event.clone());
    }
    for _ in 0..total_steps {
        recorder.finish_step(0.0);
    }
    Box::new(recorder).finish().unwrap();
    sink.contents()
}

fn temp_path(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("ynwa-core-codec-{name}-{nanos}.jsonl"))
}

#[test]
fn in_memory_record_round_trips() {
    let header = test_header();
    let entries = sample_entries();

    let buffer = write_recording(&header, &entries, 3);
    let record = JsonRecordCodec::reader(Cursor::new(buffer)).read().unwrap();

    assert_eq!(record.header, header);
    assert_eq!(record.total_steps, 3);
    assert_eq!(record.journal, entries);
}

#[test]
fn file_record_round_trips() {
    let header = test_header();
    let entries = sample_entries();
    let path = temp_path("round-trip");

    {
        let file = fs::File::create(&path).unwrap();
        let mut recorder =
            FileJournalRecorder::new(Box::new(JsonRecordCodec::writer(file)), &header);
        for entry in &entries {
            recorder.push(entry.timestamp, entry.event.clone());
        }
        for _ in 0..7 {
            recorder.finish_step(0.0);
        }
        Box::new(recorder).finish().unwrap();
    }

    let file = fs::File::open(&path).unwrap();
    let record = JsonRecordCodec::reader(BufReader::new(file))
        .read()
        .unwrap();
    fs::remove_file(&path).unwrap();

    assert_eq!(record.header, header);
    assert_eq!(record.total_steps, 7);
    assert_eq!(record.journal, entries);
}

#[test]
fn writes_header_event_and_footer_lines() {
    let header = test_header();
    let entries = sample_entries();

    let buffer = write_recording(&header, &entries, 2);
    let text = String::from_utf8(buffer).unwrap();
    let lines: Vec<&str> = text.lines().collect();

    assert_eq!(lines.len(), entries.len() + 2);
    assert!(lines[0].starts_with("{\"type\":\"header\""));
    assert!(lines[1].starts_with("{\"type\":\"event\""));
    assert_eq!(
        lines.last().unwrap(),
        &"{\"type\":\"footer\",\"total_steps\":2}"
    );
}

#[test]
fn header_is_written_on_creation() {
    let header = test_header();
    let sink = SharedBuffer::default();

    let recorder =
        FileJournalRecorder::new(Box::new(JsonRecordCodec::writer(sink.clone())), &header);

    let text = sink.text();
    assert_eq!(text.lines().count(), 1);
    assert!(text.starts_with("{\"type\":\"header\""));
    drop(recorder);
}

#[test]
fn events_are_flushed_before_finish() {
    let header = test_header();
    let entries = sample_entries();
    let path = temp_path("streaming");

    {
        let file = fs::File::create(&path).unwrap();
        let mut recorder =
            FileJournalRecorder::new(Box::new(JsonRecordCodec::writer(file)), &header);

        for (index, entry) in entries.iter().enumerate() {
            recorder.push(entry.timestamp, entry.event.clone());

            let text = fs::read_to_string(&path).unwrap();
            assert_eq!(text.lines().count(), index + 2);
        }

        Box::new(recorder).finish().unwrap();
    }

    let text = fs::read_to_string(&path).unwrap();
    fs::remove_file(&path).unwrap();

    assert_eq!(text.lines().count(), entries.len() + 2);
}

#[test]
fn reads_valid_prefix_without_footer() {
    let header = test_header();
    let entries = sample_entries();
    let complete = write_recording(&header, &entries, 5);

    let footer_start = complete
        .windows(FOOTER_MARKER.len())
        .position(|window| window == FOOTER_MARKER)
        .unwrap();
    let prefix = &complete[..footer_start];

    let record = JsonRecordCodec::reader(Cursor::new(prefix.to_vec()))
        .read()
        .unwrap();

    assert_eq!(record.header, header);
    assert_eq!(record.journal, entries);
}

#[test]
fn ignores_event_line_cut_off_mid_write() {
    let header = test_header();
    let entries = sample_entries();
    let complete = write_recording(&header, &entries, 5);

    let footer_start = complete
        .windows(FOOTER_MARKER.len())
        .position(|window| window == FOOTER_MARKER)
        .unwrap();
    let prefix = &complete[..footer_start];
    let cut = &prefix[..prefix.len() - 5];

    let record = JsonRecordCodec::reader(Cursor::new(cut.to_vec()))
        .read()
        .unwrap();

    assert_eq!(record.header, header);
    assert_eq!(record.journal, entries[..entries.len() - 1].to_vec());
}

#[test]
fn reading_without_header_fails() {
    let error = JsonRecordCodec::reader(Cursor::new(Vec::new()))
        .read()
        .unwrap_err();

    assert!(error.contains("no header"));
}

#[test]
fn duplicate_header_is_rejected() {
    let header = test_header();
    let buffer = write_recording(&header, &[], 0);
    let header_line = String::from_utf8(buffer.clone())
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_string();

    let mut duplicated = buffer;
    duplicated.extend_from_slice(header_line.as_bytes());
    duplicated.push(b'\n');

    let error = JsonRecordCodec::reader(Cursor::new(duplicated))
        .read()
        .unwrap_err();

    assert!(error.contains("more than one header"));
}

#[test]
fn corrupt_complete_line_fails() {
    let mut buffer = write_recording(&test_header(), &[], 0);
    buffer.extend_from_slice(b"{not json}\n");

    let error = JsonRecordCodec::reader(Cursor::new(buffer))
        .read()
        .unwrap_err();

    assert!(!error.is_empty());
}

struct FailingEventWriter;

impl RecordWriter for FailingEventWriter {
    fn write_header(&mut self, _header: &RecordHeader) -> Result<(), String> {
        Ok(())
    }

    fn write_event(&mut self, _entry: &JournalEntry) -> Result<(), String> {
        Err("disk full".to_string())
    }

    fn finish(&mut self, _total_steps: u64) -> Result<(), String> {
        Ok(())
    }
}

struct FailingHeaderWriter;

impl RecordWriter for FailingHeaderWriter {
    fn write_header(&mut self, _header: &RecordHeader) -> Result<(), String> {
        Err("no space for header".to_string())
    }

    fn write_event(&mut self, _entry: &JournalEntry) -> Result<(), String> {
        Ok(())
    }

    fn finish(&mut self, _total_steps: u64) -> Result<(), String> {
        Ok(())
    }
}

#[test]
fn event_write_error_is_reported_on_finish() {
    let header = test_header();
    let mut recorder = FileJournalRecorder::new(Box::new(FailingEventWriter), &header);

    recorder.push(0.1, JournalEvent::DecisionsReset);
    recorder.finish_step(0.1);

    assert_eq!(Box::new(recorder).finish().unwrap_err(), "disk full");
}

#[test]
fn header_write_error_is_reported_on_finish() {
    let header = test_header();
    let recorder = FileJournalRecorder::new(Box::new(FailingHeaderWriter), &header);

    assert_eq!(
        Box::new(recorder).finish().unwrap_err(),
        "no space for header"
    );
}

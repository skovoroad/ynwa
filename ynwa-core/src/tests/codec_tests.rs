//! Tests for the JSON Lines codec, the record I/O helpers and the file journal recorder.

use crate::codec::{FileJournalRecorder, RecordReader, RecordWriter};
use crate::field::zones::Velocity3D;
use crate::field::Field;
use crate::game::{
    BallDef, Decision, DecisionTarget, Game, GameConfig, GameStage, GameState, PlayerDef,
    RefereeDef, StatSet, REGION_START_POSITION,
};
use crate::journal::{JournalEntry, JournalEvent, JournalSink};
use crate::record::{Record, RecordFooter, RecordHeader};
use crate::record_io::{
    json_journal_file_reader, json_journal_file_writer, json_journal_memory_reader,
    json_journal_memory_writer, write_json_record, SharedBytes,
};
use crate::region::GridCell;
use crate::team::Team;
use crate::test_utils::deterministic_rng;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const FOOTER_MARKER: &[u8] = b"{\"type\":\"footer\"";
const TIMESTEP: f32 = 0.016;

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
        team_a_id: "team_a".to_string(),
        team_b_id: "team_b".to_string(),
        team_a_display_name: "Team A".to_string(),
        team_b_display_name: "Team B".to_string(),
        scripting: crate::game::ScriptingConfig::empty(),
    }
}

fn test_header() -> RecordHeader {
    let game = Game::new(test_config(), deterministic_rng());
    RecordHeader::from_game(&game, TIMESTEP)
}

/// State a playthrough of `steps` steps ends in; the footer is built from it.
fn final_state(steps: u64) -> GameState {
    let mut game = Game::new(test_config(), deterministic_rng());
    for _ in 0..steps {
        game.step(TIMESTEP);
    }
    game.state
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

fn write_recording(header: &RecordHeader, entries: &[JournalEntry], steps: u64) -> Vec<u8> {
    let sink = SharedBytes::new();
    let mut recorder =
        FileJournalRecorder::new(Box::new(json_journal_memory_writer(&sink)), header);
    for entry in entries {
        recorder.push(entry.timestamp, entry.event.clone());
    }
    Box::new(recorder).finish(&final_state(steps)).unwrap();
    sink.bytes()
}

fn temp_path(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("ynwa-core-codec-{name}-{nanos}.jsonl"))
}

/// Truncated recording = header + events, without the trailing footer line.
fn prefix_without_footer(complete: &[u8]) -> &[u8] {
    let footer_start = complete
        .windows(FOOTER_MARKER.len())
        .position(|window| window == FOOTER_MARKER)
        .unwrap();
    &complete[..footer_start]
}

#[test]
fn in_memory_record_round_trips() {
    let header = test_header();
    let entries = sample_entries();

    let buffer = write_recording(&header, &entries, 3);
    let record = json_journal_memory_reader(buffer).read().unwrap();

    assert_eq!(record.header, header);
    assert_eq!(record.footer, RecordFooter::from_state(&final_state(3)));
    assert_eq!(record.footer.total_steps, 3);
    assert_eq!(record.journal, entries);
}

#[test]
fn file_record_round_trips() {
    let header = test_header();
    let entries = sample_entries();
    let path = temp_path("round-trip");

    {
        let mut recorder =
            FileJournalRecorder::new(Box::new(json_journal_file_writer(&path).unwrap()), &header);
        for entry in &entries {
            recorder.push(entry.timestamp, entry.event.clone());
        }
        Box::new(recorder).finish(&final_state(7)).unwrap();
    }

    let mut reader = json_journal_file_reader(&path).unwrap();
    let record = reader.read().unwrap();
    fs::remove_file(&path).unwrap();

    assert_eq!(record.header, header);
    assert_eq!(record.footer.total_steps, 7);
    assert_eq!(record.journal, entries);
}

#[test]
fn write_json_record_round_trips_the_whole_record() {
    let record = Record {
        header: test_header(),
        footer: RecordFooter::from_state(&final_state(4)),
        journal: sample_entries(),
    };
    let path = temp_path("write-record");

    write_json_record(&path, &record).unwrap();

    let restored = json_journal_file_reader(&path).unwrap().read().unwrap();
    fs::remove_file(&path).unwrap();

    assert_eq!(restored, record);
}

#[test]
fn footer_round_trips_with_all_statistics() {
    let mut state = final_state(5);
    state.team_stats.clear();
    let mut scored = StatSet::default();
    scored.set("score", 2.0);
    state.team_stats.insert(Team::B, StatSet::default());
    state.team_stats.insert(Team::A, scored);
    state.player_stats[0].set("distance", 123.5);
    let record = Record {
        header: test_header(),
        footer: RecordFooter::from_state(&state),
        journal: sample_entries(),
    };
    let path = temp_path("footer-stats");

    write_json_record(&path, &record).unwrap();

    let text = fs::read_to_string(&path).unwrap();
    let restored = json_journal_file_reader(&path).unwrap().read().unwrap();
    fs::remove_file(&path).unwrap();

    assert_eq!(restored.footer, record.footer);
    assert_eq!(restored.footer.total_steps, 5);
    assert_eq!(restored.footer.elapsed_time, state.elapsed_time);
    assert_eq!(restored.footer.team_stats, state.team_stats);
    assert_eq!(restored.footer.player_stats, state.player_stats);

    let footer_line = text.lines().last().unwrap();
    assert!(
        footer_line.find("\"A\"").unwrap() < footer_line.find("\"B\"").unwrap(),
        "team_stats must be written in sorted order: {footer_line}"
    );
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
    let footer: serde_json::Value = serde_json::from_str(lines.last().unwrap()).unwrap();
    assert_eq!(footer["type"], "footer");
    assert_eq!(footer["total_steps"], 2);
    assert_eq!(
        footer["elapsed_time"].as_f64().unwrap() as f32,
        final_state(2).elapsed_time
    );
    assert_eq!(
        footer["team_stats"],
        serde_json::json!({"A": {"values": {}}})
    );
    assert_eq!(footer["player_stats"], serde_json::json!([{"values": {}}]));
}

#[test]
fn header_is_written_on_creation() {
    let header = test_header();
    let sink = SharedBytes::new();

    let recorder = FileJournalRecorder::new(Box::new(json_journal_memory_writer(&sink)), &header);

    let text = String::from_utf8(sink.bytes()).unwrap();
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
        let mut recorder =
            FileJournalRecorder::new(Box::new(json_journal_file_writer(&path).unwrap()), &header);

        for (index, entry) in entries.iter().enumerate() {
            recorder.push(entry.timestamp, entry.event.clone());

            let text = fs::read_to_string(&path).unwrap();
            assert_eq!(text.lines().count(), index + 2);
        }

        Box::new(recorder).finish(&final_state(3)).unwrap();
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

    let record = json_journal_memory_reader(prefix_without_footer(&complete).to_vec())
        .read()
        .unwrap();

    assert_eq!(record.header, header);
    assert_eq!(record.journal, entries);
    assert_eq!(record.footer, RecordFooter::default());
}

#[test]
fn ignores_event_line_cut_off_mid_write() {
    let header = test_header();
    let entries = sample_entries();
    let complete = write_recording(&header, &entries, 5);

    let prefix = prefix_without_footer(&complete);
    let cut = &prefix[..prefix.len() - 5];

    let record = json_journal_memory_reader(cut.to_vec()).read().unwrap();

    assert_eq!(record.header, header);
    assert_eq!(record.journal, entries[..entries.len() - 1].to_vec());
}

#[test]
fn reading_without_header_fails() {
    let error = json_journal_memory_reader(Vec::new()).read().unwrap_err();

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

    let error = json_journal_memory_reader(duplicated).read().unwrap_err();

    assert!(error.contains("more than one header"));
}

#[test]
fn corrupt_complete_line_fails() {
    let mut buffer = write_recording(&test_header(), &[], 0);
    buffer.extend_from_slice(b"{not json}\n");

    let error = json_journal_memory_reader(buffer).read().unwrap_err();

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

    fn finish(&mut self, _footer: &RecordFooter) -> Result<(), String> {
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

    fn finish(&mut self, _footer: &RecordFooter) -> Result<(), String> {
        Ok(())
    }
}

struct FailingFinishWriter;

impl RecordWriter for FailingFinishWriter {
    fn write_header(&mut self, _header: &RecordHeader) -> Result<(), String> {
        Ok(())
    }

    fn write_event(&mut self, _entry: &JournalEntry) -> Result<(), String> {
        Ok(())
    }

    fn finish(&mut self, _footer: &RecordFooter) -> Result<(), String> {
        Err("no space for footer".to_string())
    }
}

#[test]
fn event_write_error_is_reported_on_finish() {
    let header = test_header();
    let mut recorder = FileJournalRecorder::new(Box::new(FailingEventWriter), &header);

    recorder.push(0.1, JournalEvent::DecisionsReset);

    assert_eq!(
        Box::new(recorder).finish(&final_state(1)).unwrap_err(),
        "disk full"
    );
}

#[test]
fn header_write_error_is_reported_on_finish() {
    let header = test_header();
    let recorder = FileJournalRecorder::new(Box::new(FailingHeaderWriter), &header);

    assert_eq!(
        Box::new(recorder).finish(&final_state(0)).unwrap_err(),
        "no space for header"
    );
}

#[test]
fn finish_write_error_is_reported_on_finish() {
    let header = test_header();
    let mut recorder = FileJournalRecorder::new(Box::new(FailingFinishWriter), &header);

    recorder.push(0.1, JournalEvent::DecisionsReset);

    assert_eq!(
        Box::new(recorder).finish(&final_state(1)).unwrap_err(),
        "no space for footer"
    );
}

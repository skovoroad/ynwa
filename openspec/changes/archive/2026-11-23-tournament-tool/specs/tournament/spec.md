# Spec Delta

## Purpose

Runs a round-robin tournament over a list of teams by driving the `ynwa-simulator` binary, extracting final scores from match recordings, and reporting aggregate standings.

## ADDED Requirements

### Requirement: Tournament CLI Parameters and Defaults

The tournament runner SHALL accept, as command-line arguments: a list of team identifiers, the teams directory, the preambles directory, the simulator binary path, the number of rounds, match duration, timestep, temperature, an optional seed, the output directory, a run name, and a recording-retention flag.

#### Scenario: Valid invocation with defaults
- **WHEN** the tournament runner is invoked with a valid team list and defaults for the remaining parameters
- **THEN** it runs a tournament over those teams with a non-zero temperature and random per-match seeds

### Requirement: Tournament CLI Validation

The runner MUST reject fewer than two teams, a team identifier whose folder is missing from the teams directory, a non-positive or non-finite duration or timestep, a temperature outside `0.0..=1.0`, or a round count below `1`, printing an error to stderr and exiting non-zero.

#### Scenario: Rejecting malformed input
- **WHEN** the tournament runner is invoked with one team, a missing team folder, a non-finite duration, or `rounds < 1`
- **THEN** it prints an error to stderr and exits non-zero

### Requirement: Double Round-Robin Schedule

The tournament SHALL generate a double round-robin schedule: within one round every ordered pair `(home, away)` of distinct teams plays exactly once, and the schedule repeats this for the configured number of rounds, so every unordered pair meets an equal number of times as each side.

#### Scenario: Balanced home-and-away pairings
- **WHEN** a tournament of `N` teams and `R` rounds is scheduled
- **THEN** each round contains exactly `N * (N - 1)` matches, each ordered pair `(home, away)` appears exactly `R` times, and each unordered pair appears `2 * R` times total (equally split between the two sides)

### Requirement: Match Execution via the Simulator and Score Extraction

For each scheduled match the runner SHALL invoke the `ynwa-simulator` binary with the home team as `--team-a`, the away team as `--team-b`, the configured simulation parameters, an explicit output recording path, and a per-match seed; it SHALL then read the two final scores from the recording footer's `team_stats` `score` values and map them back to home and away. A non-zero simulator exit status SHALL abort the tournament with a non-zero exit status.

#### Scenario: Home/away score mapping and failure propagation
- **WHEN** a match is run with team `X` as home (`--team-a`) and team `Y` as away (`--team-b`)
- **THEN** the recorded `Team::A` score is credited to `X` and the `Team::B` score to `Y`
- **AND** if `ynwa-simulator` exits non-zero the tournament aborts with a non-zero exit status instead of writing a partial result

### Requirement: Reproducible Per-Match Seeding

The runner SHALL assign a fresh random seed to every match by default and MUST record the seed used for each match in the results output. When a fixed `--seed` is provided, the runner SHALL derive each match's seed deterministically from it so the entire tournament is reproducible.

#### Scenario: Random seeds are recorded and a fixed seed is deterministic
- **WHEN** a tournament runs with the default (random) seeding
- **THEN** every match row in `results.txt` includes the seed that produced it, so any single match can be re-run identically
- **AND** running the same tournament twice with the same fixed `--seed`, team list, and parameters produces identical `results.txt`

### Requirement: Release Build Before Running

Before running any matches, the tournament tool SHALL ensure the release `ynwa-simulator` binary is built, so the tournament works even when the project has only been built in debug.

#### Scenario: Missing release binary is built first
- **WHEN** the tournament tool starts
- **THEN** it ensures the release `ynwa-simulator` binary is built before the first match is executed

### Requirement: Standings Computation and Ordering

The runner SHALL accumulate, per team, matches played, wins, draws, losses, goals for, goals against, and goal difference, and SHALL assign points on a 3-point system (3 for a win, 1 for a draw, 0 for a loss). The standings SHALL be ordered by points descending, then goal difference descending, then goals for descending, then team identifier ascending.

#### Scenario: Points and deterministic tie-break ordering
- **WHEN** all matches of a tournament are complete
- **THEN** each team's points equal `3 * wins + draws`, and goals for and goals against sum across all its matches (home and away)
- **AND** teams are ordered by points, then goal difference, then goals for, then team identifier

### Requirement: Result File Output

The runner SHALL write two human-readable, space-aligned files under `<output>/<run>/`: `results.txt` (one row per match, listing round, home team, away team, home score, away score, and seed) and `standings.txt` (team identifier, played, wins, draws, losses, goals for, goals against, goal difference, and points). Team identifiers SHALL be the `teams/` folder names.

#### Scenario: Two aligned text reports
- **WHEN** a tournament completes
- **THEN** `results.txt` and `standings.txt` are written under `<output>/<run>/` with space-aligned columns, and team identifiers are the folder names rather than display names

### Requirement: Recording Retention

The runner SHALL write each match recording to a temporary location and delete it after extracting the score, unless a retention flag is set, in which case recordings SHALL be kept under `<output>/<run>/matches/`.

#### Scenario: Default discard vs retained recordings
- **WHEN** a tournament runs without the retention flag
- **THEN** no recording files remain after the run
- **AND** with the retention flag set, each match's `.jsonl` recording is kept under `<output>/<run>/matches/`

#!/usr/bin/env python3
"""Runs a double round-robin tournament over the ynwa-simulator binary."""

import argparse
import json
import math
import random
import secrets
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path

DEFAULT_TEAMS_DIR = "teams"
DEFAULT_PREAMBLES_DIR = "ynwa-scripts/preambles"
DEFAULT_SIMULATOR = "target/release/ynwa-simulator"
DEFAULT_DURATION = 120.0
DEFAULT_TIMESTEP = 1.0 / 60.0
DEFAULT_TEMPERATURE = 0.7

RESULTS_HEADER = ("round", "home", "away", "home_score", "away_score", "seed")
STANDINGS_HEADER = (
    "team",
    "played",
    "won",
    "drawn",
    "lost",
    "goals_for",
    "goals_against",
    "goal_difference",
    "points",
)


class TournamentError(Exception):
    pass


@dataclass(frozen=True)
class Match:
    round: int
    home: str
    away: str


@dataclass(frozen=True)
class MatchResult:
    match: Match
    home_score: int
    away_score: int
    seed: int


@dataclass
class Standing:
    team: str
    played: int = 0
    won: int = 0
    drawn: int = 0
    lost: int = 0
    goals_for: int = 0
    goals_against: int = 0
    goal_difference: int = 0
    points: int = 0

    def add_result(self, scored: int, conceded: int) -> None:
        self.played += 1
        self.goals_for += scored
        self.goals_against += conceded
        self.goal_difference = self.goals_for - self.goals_against
        if scored > conceded:
            self.won += 1
        elif scored == conceded:
            self.drawn += 1
        else:
            self.lost += 1
        self.points = 3 * self.won + self.drawn


def positive_finite(value: str) -> float:
    try:
        number = float(value)
    except ValueError as error:
        raise argparse.ArgumentTypeError(f"must be a number, got '{value}'") from error
    if not math.isfinite(number) or number <= 0.0:
        raise argparse.ArgumentTypeError(f"must be a positive finite number, got '{value}'")
    return number


def temperature(value: str) -> float:
    try:
        number = float(value)
    except ValueError as error:
        raise argparse.ArgumentTypeError(f"must be a number, got '{value}'") from error
    if not math.isfinite(number) or not 0.0 <= number <= 1.0:
        raise argparse.ArgumentTypeError(f"must be in [0.0, 1.0], got '{value}'")
    return number


def rounds(value: str) -> int:
    try:
        number = int(value)
    except ValueError as error:
        raise argparse.ArgumentTypeError(f"must be an integer, got '{value}'") from error
    if number < 1:
        raise argparse.ArgumentTypeError(f"must be at least 1, got '{value}'")
    return number


def seed(value: str) -> int:
    try:
        number = int(value)
    except ValueError as error:
        raise argparse.ArgumentTypeError(f"must be an integer, got '{value}'") from error
    if not 0 <= number <= 2**64 - 1:
        raise argparse.ArgumentTypeError(f"must be a u64, got '{value}'")
    return number


def parse_args(argv):
    parser = argparse.ArgumentParser(
        description="Run a double round-robin tournament over ynwa-simulator.",
    )
    parser.add_argument(
        "team_ids",
        nargs="+",
        metavar="TEAM",
        help="team folder names under --teams (at least two)",
    )
    parser.add_argument(
        "--teams",
        dest="teams_dir",
        default=DEFAULT_TEAMS_DIR,
        help=f"directory with team folders (default: {DEFAULT_TEAMS_DIR})",
    )
    parser.add_argument(
        "--preambles",
        dest="preambles_dir",
        default=DEFAULT_PREAMBLES_DIR,
        help=f"directory with Lua preambles (default: {DEFAULT_PREAMBLES_DIR})",
    )
    parser.add_argument(
        "--simulator",
        default=DEFAULT_SIMULATOR,
        help=f"ynwa-simulator binary (default: {DEFAULT_SIMULATOR})",
    )
    parser.add_argument(
        "--rounds",
        type=rounds,
        default=1,
        help="number of round-robin rounds, each containing all ordered pairs (default: 1)",
    )
    parser.add_argument(
        "--duration",
        type=positive_finite,
        default=DEFAULT_DURATION,
        help=f"match duration in seconds (default: {DEFAULT_DURATION})",
    )
    parser.add_argument(
        "--timestep",
        type=positive_finite,
        default=DEFAULT_TIMESTEP,
        help=f"simulation timestep in seconds (default: {DEFAULT_TIMESTEP})",
    )
    parser.add_argument(
        "--temperature",
        type=temperature,
        default=DEFAULT_TEMPERATURE,
        help=f"decision temperature in [0, 1] (default: {DEFAULT_TEMPERATURE})",
    )
    parser.add_argument(
        "--seed",
        type=seed,
        default=None,
        help="base seed making the whole tournament reproducible (default: random per match)",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path.home() / ".ynwa" / "tournaments",
        help="root output directory (default: ~/.ynwa/tournaments)",
    )
    parser.add_argument(
        "--name",
        default=None,
        help="run directory name (default: current timestamp)",
    )
    parser.add_argument(
        "--keep-recordings",
        action="store_true",
        help="keep match recordings under <run>/matches instead of discarding them",
    )

    args = parser.parse_args(argv)
    validate(parser, args)
    return args


def validate(parser, args) -> None:
    if len(args.team_ids) < 2:
        parser.error("at least two teams are required")

    teams_dir = Path(args.teams_dir)
    if not teams_dir.is_dir():
        parser.error(f"teams directory not found: {teams_dir}")

    missing = [team for team in args.team_ids if not (teams_dir / team).is_dir()]
    if missing:
        parser.error(f"team folder(s) not found under {teams_dir}: {', '.join(missing)}")

    if not Path(args.simulator).is_file():
        parser.error(f"simulator binary not found: {args.simulator}")


def build_schedule(team_ids, round_count):
    """One round holds every ordered pair of distinct teams; rounds repeat it."""
    return [
        Match(round=round_number, home=home, away=away)
        for round_number in range(1, round_count + 1)
        for home in team_ids
        for away in team_ids
        if home != away
    ]


def match_seeds(schedule, base_seed):
    if base_seed is None:
        return [secrets.randbits(64) for _ in schedule]
    generator = random.Random(base_seed)
    return [generator.getrandbits(64) for _ in schedule]


def run_simulator(args, match, match_seed, recording_path):
    command = [
        args.simulator,
        "--teams",
        args.teams_dir,
        "--preambles",
        args.preambles_dir,
        "--team-a",
        match.home,
        "--team-b",
        match.away,
        "--duration",
        str(args.duration),
        "--timestep",
        str(args.timestep),
        "--temperature",
        str(args.temperature),
        "--seed",
        str(match_seed),
        "--out",
        str(recording_path),
    ]
    completed = subprocess.run(command, capture_output=True, text=True, check=False)
    if completed.returncode != 0:
        details = completed.stderr.strip()
        message = (
            f"{match.home} vs {match.away}: ynwa-simulator exited with "
            f"status {completed.returncode}"
        )
        if details:
            message = f"{message}: {details}"
        raise TournamentError(message)


def read_score(recording_path):
    """Reads the final `A`/`B` scores from the recording footer."""
    lines = [
        line
        for line in recording_path.read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    if not lines:
        raise TournamentError(f"recording is empty: {recording_path}")
    try:
        footer = json.loads(lines[-1])
    except json.JSONDecodeError as error:
        raise TournamentError(f"recording footer is not JSON: {recording_path}") from error
    try:
        home_values = footer["team_stats"]["A"]["values"]
        away_values = footer["team_stats"]["B"]["values"]
        home_score = int(home_values.get("score", 0))
        away_score = int(away_values.get("score", 0))
    except (KeyError, TypeError, ValueError) as error:
        raise TournamentError(
            f"recording footer has no team_stats score: {recording_path}"
        ) from error
    return home_score, away_score


def play_match(args, match, match_seed, recording_path):
    run_simulator(args, match, match_seed, recording_path)
    return read_score(recording_path)


def play_tournament(args, schedule, seeds, run_dir):
    results = []
    for index, (match, match_seed) in enumerate(zip(schedule, seeds), start=1):
        if args.keep_recordings:
            matches_dir = run_dir / "matches"
            matches_dir.mkdir(parents=True, exist_ok=True)
            recording_path = (
                matches_dir / f"{index:04d}_{match.home}_vs_{match.away}.jsonl"
            )
            home_score, away_score = play_match(
                args, match, match_seed, recording_path
            )
        else:
            with tempfile.TemporaryDirectory() as temporary_dir:
                recording_path = Path(temporary_dir) / "match.jsonl"
                home_score, away_score = play_match(
                    args, match, match_seed, recording_path
                )

        result = MatchResult(match, home_score, away_score, match_seed)
        results.append(result)
        print(
            f"Round {match.round}: {match.home} {home_score}-{away_score} "
            f"{match.away} (seed {match_seed})"
        )
    return results


def compute_standings(team_ids, results):
    table = {team: Standing(team) for team in team_ids}
    for result in results:
        home = table[result.match.home]
        away = table[result.match.away]
        home.add_result(result.home_score, result.away_score)
        away.add_result(result.away_score, result.home_score)
    return sorted(
        table.values(),
        key=lambda standing: (
            -standing.points,
            -standing.goal_difference,
            -standing.goals_for,
            standing.team,
        ),
    )


def format_table(header, rows):
    columns = [header, *rows]
    widths = [
        max(len(str(column[index])) for column in columns)
        for index in range(len(header))
    ]
    lines = []
    for column in columns:
        cells = [
            str(value).ljust(widths[index]) if index == 0 else str(value).rjust(widths[index])
            for index, value in enumerate(column)
        ]
        lines.append("  ".join(cells).rstrip())
    return "\n".join(lines) + "\n"


def write_reports(run_dir, results, standings):
    result_rows = [
        (
            result.match.round,
            result.match.home,
            result.match.away,
            result.home_score,
            result.away_score,
            result.seed,
        )
        for result in results
    ]
    standings_rows = [
        (
            standing.team,
            standing.played,
            standing.won,
            standing.drawn,
            standing.lost,
            standing.goals_for,
            standing.goals_against,
            standing.goal_difference,
            standing.points,
        )
        for standing in standings
    ]
    (run_dir / "results.txt").write_text(
        format_table(RESULTS_HEADER, result_rows), encoding="utf-8"
    )
    (run_dir / "standings.txt").write_text(
        format_table(STANDINGS_HEADER, standings_rows), encoding="utf-8"
    )


def main(argv=None):
    args = parse_args(argv)
    run_name = args.name or datetime.now().strftime("%Y%m%d-%H%M%S")
    run_dir = (Path(args.output).expanduser() / run_name).resolve()
    run_dir.mkdir(parents=True, exist_ok=True)

    schedule = build_schedule(args.team_ids, args.rounds)
    seeds = match_seeds(schedule, args.seed)

    try:
        results = play_tournament(args, schedule, seeds, run_dir)
    except TournamentError as error:
        print(f"Error: {error}", file=sys.stderr)
        return 1

    standings = compute_standings(args.team_ids, results)
    write_reports(run_dir, results, standings)
    print(f"Results: {run_dir / 'results.txt'}")
    print(f"Standings: {run_dir / 'standings.txt'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

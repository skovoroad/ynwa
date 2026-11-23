#!/usr/bin/env bash
# Runs a double round-robin tournament over the teams configured below.
# Edit the variables to change the tournament parameters.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

TEAMS=(team_a team_b team_c)
ROUNDS=10
DURATION=120.0
TIMESTEP=0.0166666667
TEMPERATURE=0.7
OUTPUT="$HOME/.ynwa/tournaments"
RUN_NAME="tournament"
KEEP_RECORDINGS=0

cargo build --release --bin ynwa-simulator

ARGS=(
    --teams teams
    --preambles ynwa-scripts/preambles
    --simulator target/release/ynwa-simulator
    --rounds "$ROUNDS"
    --duration "$DURATION"
    --timestep "$TIMESTEP"
    --temperature "$TEMPERATURE"
    --output "$OUTPUT"
    --name "$RUN_NAME"
)
if [ "$KEEP_RECORDINGS" -eq 1 ]; then
    ARGS+=(--keep-recordings)
fi

python3 tools/tournament.py "${ARGS[@]}" "${TEAMS[@]}"

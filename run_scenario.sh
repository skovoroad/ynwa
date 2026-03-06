#!/usr/bin/env bash
# Запускает сценарий: передаёт путь к тактикам сценария вместо стандартных teams/
# Использование: ./run_scenario.sh <имя_сценария>
# Пример:        ./run_scenario.sh goal_kick_teamA_left

set -e

SCENARIO="${1:?Укажи имя сценария, например: goal_kick_teamA_left}"
REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"
TEAMS_PATH="$REPO_ROOT/ynwa-script-tests/scenarios/$SCENARIO/teams"
PREAMBLES_PATH="$REPO_ROOT/ynwa-scripts/preambles"

if [ ! -d "$TEAMS_PATH" ]; then
    echo "Сценарий не найден: $TEAMS_PATH"
    echo "Доступные сценарии:"
    ls "$REPO_ROOT/ynwa-script-tests/scenarios/"
    exit 1
fi

cd "$REPO_ROOT"
cargo run --release --bin ynwa-player -- "$TEAMS_PATH" "$PREAMBLES_PATH"

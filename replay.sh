#!/usr/bin/env bash
# Воспроизводит запись матча или сценария.
# Использование: ./replay.sh <путь_к_записи.jsonl>
# Пример:        ./replay.sh ~/.ynwa/scenarios/stdlib_kick_to_goal.jsonl

set -e

RECORDING="${1:?Укажи путь к записи, например: ~/.ynwa/scenarios/stdlib_kick_to_goal.jsonl}"

if [ ! -f "$RECORDING" ]; then
    echo "Запись не найдена: $RECORDING"
    exit 1
fi

# Абсолютный путь, чтобы запись открылась после перехода в корень репозитория.
RECORDING_DIR="$(cd "$(dirname "$RECORDING")" && pwd)"
RECORDING="$RECORDING_DIR/$(basename "$RECORDING")"

REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$REPO_ROOT"

cargo run --release --bin ynwa-player -- --replay "$RECORDING"

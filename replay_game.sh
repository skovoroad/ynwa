#!/usr/bin/env bash
# Воспроизводит последнюю записанную игру.
# Использование: ./replay_game.sh
# Каталог записей: ~/.ynwa/games (переопределяется переменной YNWA_GAMES_DIR).

set -e

REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"
GAMES_DIR="${YNWA_GAMES_DIR:-$HOME/.ynwa/games}"

LATEST="$(ls -1t "$GAMES_DIR"/game_*.jsonl 2>/dev/null | head -n 1 || true)"
if [ -z "$LATEST" ]; then
    echo "Записи не найдены в $GAMES_DIR"
    echo "Сначала запусти ./record_game.sh"
    exit 1
fi

echo "Воспроизведение: $LATEST"
cd "$REPO_ROOT"

cargo run --release --bin ynwa-player -- --replay "$LATEST"

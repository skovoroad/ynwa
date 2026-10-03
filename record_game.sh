#!/usr/bin/env bash
# Запускает дефолтную игру (teams/, ynwa-scripts/preambles) с записью матча.
# Использование: ./record_game.sh [доп. флаги плеера, например: --out /tmp/games]
# По умолчанию запись идёт в ~/.ynwa/games (переопределяется --out).

set -e

REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$REPO_ROOT"

cargo run --release --bin ynwa-player -- --record "$@"

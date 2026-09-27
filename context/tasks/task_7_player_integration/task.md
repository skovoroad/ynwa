# Задача

Интегрировать запись/воспроизведение в плеер (`ynwa-player`): CLI-режимы `--record` и `--replay`.

Выделено из задачи сериализации (`task_3_game_serialization`). Возможности записи и воспроизведения
реализуются и проверяются в ядре без плеера.

# Итоги обсуждения — предложения

## 1. Интеграция в плеер

- Режим записи: `ynwa-player <teams> <preambles> --record <file>` — обычная игра с фиксированным
  шагом, `game.set_recording(true)`, `Recorder::new(fixed_dt)` считает шаги; по завершении матча —
  `JsonRecordCodec::encode` → файл.
- Режим воспроизведения: `ynwa-player --replay <file>` — `JsonRecordCodec::decode` +
  `create_football_replay_world`, шаг `fixed_dt`; управление паузой/скоростью через существующий
  [`SimulationControl`](ynwa-player/src/simulation.rs:2) (скорость влияет только на темп подачи
  шагов, сам размер шага всегда `fixed_dt` из записи).

## 2. Требуемые доработки

- CLI-аргументы `--record <file>` / `--replay <file>` в [`ynwa-player`](ynwa-player/src/main.rs:28).
- Режим записи: сбор `Record` через `Recorder` и запись через `JsonRecordCodec`.
- Режим воспроизведения: чтение через `JsonRecordCodec`, построение replay-мира
  (`create_football_replay_world`), шаг по `fixed_dt`.

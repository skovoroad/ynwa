# Задача

Реализовать снапшоты полного состояния игры в записи и их использование при воспроизведении.

У нас есть частичные снапшоты для целей интеграционного тестирования, реализованного в модуле ynwa-integration-testing. Они позволяют скорректировать начальное состояние игры и проверить конечное состояние игры при прогоне тестового сценария.

Однако мы хотим развить их использование:

1. Проигрывание сценариев интеграционного тестирования в плеере. Симуляция должна записываться средствами ядра (они есть) так, чтобы потом воспроизвестись в плеере. Но в записи игры нет начального состояния, только начальная расстановка команд, а в тестовом сценарии она может быть скорректирована.
2. Перемотка в плеере.
3. Коррекция численного дрейфа при проигрывании сценария (т.к. физика пересимулируется)

Наша цель — реализовать поддержку снапшотов для цели 1, так, чтобы в будущем можно было реализовать цели 2 и 3.

Для цели 1 достаточно:

- сериализовать полное состояние игры (`GameState`) и класть в запись снапшот начального состояния;
- применять этот снапшот при создании replay-мира, чтобы воспроизведение стартовало из того же состояния, что и сценарий (стадия, позиции/скорости игроков и мяча, владение, счёт, `restart_*`);
- чтобы прогон сценария `ynwa-integration-testing` записывал результат в файл формата записи (JSON Lines), а плеер открывал его в режиме Replay.

DOD:

- механизм снапшотов и записи игры доработан так, чтобы сценарии интеграционного тестирования можно было проверить визуально, запуская их записи в плеере;
- результат проверяется автоматически: replay-мир, стартующий из снапшота начального состояния, эквивалентен оригинальному прогону сценария (lockstep-сравнение, как в существующих replay-тестах).

Примечание для архитектурного решения: в `ynwa-core` уже есть тип `PartialStateSpec` (частичная спецификация состояния для интеграционного тестирования). Полное состояние в записи представлено самим `GameState`, поэтому конфликта имён нет.

---

# Архитектурное решение

## 1. Общая схема

В запись (`Record`) добавляется полное начальное состояние игры. Replay-мир строит начальное
состояние в те же две фазы, что и сценарий: сначала из `config` собирается `Game`, затем поверх
применяется снапшот начального состояния из записи. `config` остаётся в заголовке (см. ниже) —
из него строится `Game` и берутся статические данные для replay-физики. Это закрывает цель 1
задачи: прогон сценария `ynwa-integration-testing` (где начальное состояние скорректировано
частичным `PartialStateSpec`) записывается в файл JSON Lines, а плеер открывает его в `--replay`
и видит ровно тот же старт сценария.

Полным состоянием в записи является сам `GameState` — он получает serde. Отдельного типа
«снапшота записи» не вводим: частичный `PartialStateSpec` остаётся для сценариев интеграционного
тестирования, а в записи лежит сериализованный [`GameState`](ynwa-core/src/game.rs:261). Тем самым
конфликт имён исключён, а для будущих целей 2–3 (перемотка, коррекция дрейфа) тот же `GameState`
можно будет класть в запись и на произвольном шаге.

[`RecordHeader`](ynwa-core/src/record.rs:9) меняется: `{ config, fixed_dt, initial_state: GameState }`
вместо `{ config, initial_stage, fixed_dt }`. Стадия содержится в `initial_state.stage`, отдельное
поле `initial_stage` удаляется — одно поле правды, невозможность их расхождения.

`config` в заголовке остаётся: из него replay-мир строит `Game` и берёт статические данные
(`PlayerDef`, `Field`, скрипты), а `ReplayDriver` — характеристики игроков (`speed_rate`) для
расчёта физики. Снапшот заменяет только *начальное состояние* (`state`), а не конфигурацию матча.

Поток для сценария:

1. `ScenarioLoader` собирает мир и применяет частичный `PartialStateSpec` (как сейчас).
2. [`run()`](ynwa-integration-testing/src/runner.rs:34) в начале захватывает
   `RecordHeader::from_game(world.game(), run.dt)` — это и есть полный снапшот начального состояния
   сценария.
3. Прогон пишет журнал в `CollectJournalRecorder` (как сейчас).
4. После прогона `run()` собирает `Record` и возвращает его в `RunOutcome`.
5. `run_scenario()` сериализует `Record` в `ynwa-integration-testing/recordings/<scenario>.jsonl`
   (JSON Lines) и возвращает `ScenarioReport` (как сейчас).
6. Плеер открывает файл в `--replay`: `create_football_replay_world` применяет `initial_state` и
   воспроизводит сценарий.

`ynwa-player` не требует изменений: `--replay <file>` уже читает `Record` и строит replay-мир через
`create_football_replay_world`; после правок этапов 1–2 он автоматически стартует из `initial_state`.
Останется только вручную проверить открытие записи сценария.

## Этап 1. `ynwa-core`: сериализация состояния и модель записи

Цель: сделать `GameState` сериализуемым, класть его в `RecordHeader`, уметь применять полное
состояние и записывать готовый `Record` в файл.

Изменения:

- [`game.rs`](ynwa-core/src/game.rs:1):
  - [`PlayerState`](ynwa-core/src/game.rs:154), [`BallState`](ynwa-core/src/game.rs:183),
    [`RefereeState`](ynwa-core/src/game.rs:207) и [`GameState`](ynwa-core/src/game.rs:261) получают
    `Serialize, Deserialize` и `PartialEq` (все поля уже serde-совместимы: `Point3D`, `Velocity3D`,
    `Decision`, `Team`, `GameStage`). `PartialEq` нужен, чтобы `RecordHeader`/`Record` остались
    `PartialEq`.
  - [`StatSet`](ynwa-core/src/game.rs:31) получает `Serialize, Deserialize`; его `values`
    сериализуется детерминированно через `sorted_collections::map`.
  - `GameState.team_stats: HashMap<Team, StatSet>` сериализуется детерминированно (сортировка по
    `Team`).
  - `sorted_collections::map` обобщается на `K: Ord + Serialize` (сейчас ключ фиксирован `String`),
    чтобы покрыть `HashMap<Team, StatSet>`; `sorted_collections::set` остаётся как есть.
  - Новый метод `Game::apply_state(&mut self, state: GameState) -> Result<(), String>`: проверяет,
    что длины `player_states`/`player_stats`/`referee_states` совпадают с `config` и что
    `ball_state.possessed_by` в границах, затем заменяет `self.state`.
- [`record.rs`](ynwa-core/src/record.rs:9): `RecordHeader { config, fixed_dt, initial_state }`;
  `RecordHeader::from_game` захватывает `game.state().clone()`.
- [`record_io.rs`](ynwa-core/src/record_io.rs:1): новый
  `write_json_record(path, &Record) -> Result<(), String>` — открывает `JsonRecordWriter`, пишет
  header, все события, footer.

DOD: `cargo test -p ynwa-core` зелёный; round-trip тесты `GameState` и вложенных типов,
детерминированности `StatSet`/`team_stats`, round-trip `Record` через JSON Lines; тесты
`Game::apply_state` (валидное состояние и ошибки валидации индексов).

## Этап 2. `ynwa-football`: replay из снапшота

Цель: `create_football_replay_world` применяет `initial_state` из записи.

Изменения:

- [`replay.rs::create_football_replay_world`](ynwa-football/src/replay.rs:42): строит
  `Game::new(record.header.config, rng)`, затем `game.apply_state(record.header.initial_state)?`;
  набор систем не меняется (`ReplaySetupBallPlacer` → `ReplayDriver` → `PhysicsSystem`).
  Это повторяет настройку сценария в две фазы: сначала мир собирается из конфигурации (сценарий —
  из настроек команд), затем поверх применяется снапшот. Разница в том, что снапшот сценария
  частичный и сливается поверх, а снапшот записи полный (`GameState`) — это уже результат обеих
  фаз сценария, поэтому `apply_state` заменяет состояние целиком, а не сливает отдельные поля.
- Обновить [`replay_tests.rs`](ynwa-football/src/tests/replay_tests.rs:1) и
  [`lockstep_tests.rs`](ynwa-football/src/tests/lockstep_tests.rs:1) под новый `RecordHeader`
  (поле `initial_state` вместо `initial_stage`).
- Новый lockstep-тест: прогон с заведомо нестандартным начальным состоянием (мяч в углу с начальной
  скоростью, игроки с начальными скоростями) записывается с `initial_state`; replay стартует из
  него и пошагово совпадает с оригиналом (`run_lockstep` + `assert_equivalent`).

DOD: `cargo test -p ynwa-football` зелёный; replay-тесты проходят; lockstep-тест подтверждает
эквивалентность при нестандартном старте.

## Этап 3. `ynwa-integration-testing`: запись прогона в файл и проверка

Цель: прогон сценария оставляет JSON Lines-запись, читаемую плеером; автоматическая проверка
«запись → replay → эквивалентность».

Изменения:

- [`runner.rs`](ynwa-integration-testing/src/runner.rs:22): `RunOutcome` вместо
  `journal: Vec<JournalEntry>` содержит `record: Record` (в нём и journal, и header с
  `initial_state`). В начале `run()` захватывается `RecordHeader::from_game(world.game(), run.dt)`;
  после цикла — `finish_journal` и `collection.take_record(header)`.
- [`compare.rs::compare_outcome`](ynwa-integration-testing/src/compare.rs:35): журнал берётся из
  `outcome.record.journal`.
- [`scenario.rs::run_scenario`](ynwa-integration-testing/src/scenario.rs:19): после `run(...)`
  пишет `outcome.record` в `ynwa-integration-testing/recordings/<scenario>.jsonl` через
  `write_json_record` (каталог создаётся при необходимости; запись выполняется до сверки ожиданий,
  поэтому файл есть и для упавшего сценария).
- `.gitignore`: добавить `ynwa-integration-testing/recordings/`.
- Тесты:
  - юнит: `run()` возвращает `Record`, у которого `header.initial_state` соответствует начальному
    состоянию мира, а `journal`/`total_steps` — прогону;
  - юнит: файл записи читается обратно `json_journal_file_reader` в `Record` с тем же
    `initial_state`/`journal`/`total_steps`;
  - интеграционный: для реального сценария (например, `stdlib_kick_to_goal`) запись из
    `run_scenario` воспроизводится `create_football_replay_world` и даёт состояние, эквивалентное
    оригинальному прогону (сравнение с допуском `TOLERANCE` по контракту replay, локальный хелпер
    в тестах).

DOD: `cargo test -p ynwa-integration-testing` зелёный; в `recordings/` появляются JSON Lines-файлы
сценариев; запись читается плеером в `--replay` и replay эквивалентен прогону.

## Проверка DOD

- сценарий записывается в JSON Lines и открывается в плеере `--replay` — Этап 3 и примечание в
  «Общей схеме»;
- автоматическая проверка эквивалентности replay из снапшота оригиналу — lockstep-тест Этапа 2
  (механизм ядра/футбола) и интеграционный тест Этапа 3 (связка «сценарий → запись → replay»).


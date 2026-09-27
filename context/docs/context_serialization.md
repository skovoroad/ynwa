# Запись и воспроизведение матча

Контекст по сериализации результатов обсчёта игры (task_3). Цели: запись/воспроизведение матча
(игра, отладка) и интеграционные тесты по журналу событий. Не реализовано и вынесено в отдельные
задачи: снапшоты/перемотка (`task_4`), версионирование формата (`task_5`), кроссплатформенный
детерминизм (`task_6`), интеграция в плеер (`task_7`).

## 1. Модель

**Событийно-управляемая пересимуляция физики.** Покадровая запись не ведётся: в журнал пишутся
только существенные события с таймстампами (абсолютное время шага — аргумент `System::update`).
При воспроизведении Lua и системы принятия решений не запускаются — события подаются из журнала,
заново считается только физика.

- **RNG пишется исходами**, а не сырыми значениями: смена владения и фактическая скорость мяча
  после удара. Replay не зависит от RNG.
- **Шаг фиксированный**: в запись кладутся `fixed_dt` и `total_steps`.
- **Модель записи отделена от кодирования**: `Record` — in-memory модель, байты — за трейтами
  `RecordWriter`/`RecordReader`.
- Детерминизм гарантируется в рамках одной платформы. Serde-вывод стабилен: `HashMap`/`HashSet`
  сериализуются в отсортированном виде (`sorted_collections` в `game.rs`), зоны `Field` — в
  детерминированном порядке.

## 2. Serde

`Serialize`/`Deserialize` есть у конфигурационных типов ядра (`GameConfig`, `PlayerDef`, `Field`,
зоны, `Region`/`GridCell`, `Team`, `GameStage`, `Decision`, `Point3D`/`Velocity3D` и др.), у
`JournalEntry`/`JournalEvent` и у `FootballEvent`. Типы состояния (`GameState` и вложенные) **не**
сериализуются — это задача снапшотов.

## 3. Журнал (`ynwa-core/src/journal.rs`)

Журнал — единственный канал, через который игра сообщает о значимых событиях. Системы только
сообщают о событии; что с ним сделать (выбросить, накопить, записать в файл), решает подключённый
приёмник. Поэтому код систем не зависит от способа хранения.

- `JournalEntry { timestamp, event }`; `JournalEvent`: `DecisionAssigned`, `PossessionChange`,
  `KickOutcome`, `StageChange`, `RestartSet`, `DecisionsReset`, `StatUpdate`,
  `External { kind, data }` (спорт-специфичные события; ядро их не интерпретирует, футбол кладёт
  туда `FootballEvent` с `kind = "football_event"`).
- `trait JournalSink { push, finish_step, finish }` — чистый приёмник; куда девать события, решает
  реализация. Шаги считает сам приёмник по `finish_step`.
- Реализации: `NullJournalSink` (по умолчанию), `CollectJournalRecorder` (пишет во внешнюю
  `Rc<RefCell<EventsCollection>>`; `EventsCollection::take_record(header)` собирает `Record`),
  `FileJournalRecorder` (`codec.rs`, потоковая запись с flush каждой строки; ошибки записи копятся
  и возвращаются из `finish`).
- `Game` владеет приёмником: `set_journal_sink`, `record(timestamp, event)`, `finish_step`,
  `finish_journal()` (отцепляет sink и финализирует). `World::step` в конце шага вызывает
  `Game::finish_step`.

Точки записи: `DecisionSystem` (`DecisionAssigned`, в display-ориентации),
`BallPossessionSystem` (`PossessionChange`), `ActionSystem` (`KickOutcome`),
`FootballGameManager` (футбольное событие как `External`, затем его эффекты: `StageChange`,
`RestartSet`, `DecisionsReset`, `StatUpdate`; решения расстановки `Setup` — `DecisionAssigned`).

## 4. Запись и формат

Запись — это заголовок (всё, что нужно, чтобы заново собрать мир) плюс журнал и число шагов.
Формат байтов вынесен за трейты, чтобы позже можно было добавить бинарный кодек без изменения
модели.

- `record.rs`: `RecordHeader { config, initial_stage, fixed_dt }` (`RecordHeader::from_game`),
  `Record { header, total_steps, journal }`.
- `codec.rs`: `RecordWriter`/`RecordReader`, JSON Lines-реализации `JsonRecordWriter`/
  `JsonRecordReader`. Строки: `header`, `event`…, `footer { total_steps }`. Оборванный файл
  (без футера) читается как валидный префикс.
- `record_io.rs`: конструкторы `json_journal_file_writer/reader` и
  `json_journal_memory_writer/reader` (`SharedBytes` — буфер в памяти).

Типичное использование:

```rust
let collection = Rc::new(RefCell::new(EventsCollection::default()));
let header = RecordHeader::from_game(world.game(), dt);
world.game_mut().set_journal_sink(Box::new(CollectJournalRecorder::new(collection.clone())));
// world.step(dt) × N
world.game_mut().finish_journal()?;
let record = collection.borrow_mut().take_record(header);
```

## 5. Воспроизведение

Для воспроизведения собирается отдельный мир: в нём нет Lua и систем принятия решений, а решения и
исходы случайных веток берутся из журнала. Физика при этом та же, что и в оригинале, поэтому
результат совпадает с исходным прогоном.

- `ynwa-core/src/replay.rs`: `ReplayDriver` — система, применяющая записи журнала с
  `timestamp <= текущий` по курсору. Расчёт скорости по решению общий с `ActionSystem`
  (`systems/movement.rs`, `pub(crate)`), чтобы физика совпадала.
- `ynwa-football/src/replay.rs`:
  - `create_football_replay_world(record)` — мир с сокращённым набором систем:
    `ReplaySetupBallPlacer` (пин мяча в `Setup`, как в менеджере) → `ReplayDriver` →
    `PhysicsSystem`. Клиент делает `world.step(fixed_dt)` ровно `total_steps` раз.
  - `decode_football_events(&[JournalEntry])` — извлечение `FootballEvent` из `External`.

**Контракт эквивалентности replay**: совпадают позиции/скорости игроков и мяча, `possessed_by`,
`last_possessing_team`, `last_possession_change_time`, `stage`, `team_stats`, `restart_*`,
`elapsed_time`. Координационные флаги игроков (`needs_decision`, `is_ready`, `last_error`, …) не
гарантируются.

## 6. Тесты

Инфраструктура в `ynwa-football/src/test_utils.rs` (`assert_equivalent`, lockstep-прогон).
Lockstep-тесты сравнивают оригинал и replay на каждом шаге; `check_events` снимается **до** шага
и только в стадии `Play` (менеджер детектирует события первой системой на состоянии предыдущего
тика). Покрыты: сквозной путь через файл, независимость от RNG, борьба за мяч.

# Задача

Необходимо реализовать сериализацию/десериализацию результатов обсчёта игры.

Цели (по приоритету):

1. Записать и воспроизвести игру — игровая цель и отладка
2. Интеграционное тестирование: игра обсчитывается с записью журнала, проверяется ожидаемая последовательность событий.

Объём минимального решения: запись и воспроизведение «с нуля» на уровне ядра. Снапшоты (перемотка),
версионирование, кроссплатформенный детерминизм и интеграция в плеер вынесены в отдельные задачи
(`task_4_serialization_snapshots`, `task_5_serialization_versions`,
`task_6_serialization_crossplatform`, `task_7_player_integration`). В будущем возможна сетевая трансляция, сейчас временно исключена из рассмотрени.

# Детали

## 1. Модель воспроизведения

Предлагается **событийно-управляемая пересимуляция физики**:

- При записи в журнал пишутся только существенные события (решения, исходы случайных веток,
  футбольные события, смены стадий) с таймстампами. Покадровая запись не нужна.
- При воспроизведении движок **не перезапускает** Lua и системы принятия решений
  ([`DecisionSystem`](ynwa-core/src/systems/decision/decision_system.rs:124),
  [`BallPossessionSystem`](ynwa-core/src/systems/ball_possession.rs:97),
  [`ActionSystem`](ynwa-core/src/systems/action.rs:79)): события подаются из журнала по
  таймстампам, а заново прогоняется только физическая часть
  ([`PhysicsSystem`](ynwa-core/src/systems/physics.rs:28)): бег игроков, полёт мяча с трением,
  прилипание мяча к владельцу.

Следствие: воспроизведение точно повторяет исходный прогон при малом объёме записи.

## 2. Состав записи

Запись состоит из:

- **Заголовок**: полный [`GameConfig`](ynwa-core/src/game.rs:228) (поле, сетка, зоны, игроки,
  преамбулы и скрипты) + начальная стадия. Конфиг неизменяем во время матча, поэтому сериализуется
  один раз.
- **Журнал событий** (§3).
- **Фиксированный шаг** и число шагов — нужны для физической интеграции при воспроизведении.

## 3. Журнал событий

Каждое событие имеет таймстамп — абсолютное время шага, передаваемое системам аргументом в
[`World::step`](ynwa-core/src/world.rs:23). Предлагаемый набор событий:

- Решения игроков (`Stop` / `Run` / `Kick`) — вместо повторного вызова Lua.
- Исходы случайных веток: смена владения мячом, результат удара (фактическая скорость мяча после
  разброса).
- Футбольные события ([`FootballEvent`](ynwa-football/src/events.rs:8): гол, аут, угловой/удар от
  ворот, конец матча) и смены стадий.
- Сбросы решений, выполняемые менеджером игры при переходах в `Setup`.

Журнал должен быть доступен всем системам, включая футбольный слой. Предлагается буфер внутри
[`Game`](ynwa-core/src/game.rs:254) (по аналогии с `rng_manager`) с методом записи; события
таймстампятся аргументом `timestamp`.

## 4. Случайность (RNG)

Предлагается писать в журнал **исходы** случайных веток, а не сырые значения генератора:
потребителей случайности в игре всего два — борьба за мяч
([`select_winner`](ynwa-core/src/systems/ball_possession.rs:43)) и разброс удара
([`ActionSystem::Kick`](ynwa-core/src/systems/action.rs:109)). Это делает воспроизведение
независимым от порядка розыгрышей и внутренностей RNG.

## 5. Детерминизм

- Для v1: гарантировать детерминизм в рамках одной платформы (задокументировать).
- Упорядочить итерации по `HashMap` (зоны — [`Field.zones()`](ynwa-core/src/field/mod.rs:114),
  статистика) для стабильного вывода сериализации; для логики с «первым совпадением» использовать
  детерминированный порядок.

## 6. Начальное состояние

- Для тестов достаточно начальных позиций с точностью до региона/стандартного ключа и начальных
  заданий — это выразимо уже сейчас через конфигурацию.

---

# Архитектурное решение (шаг 2)

> ✅ Ревью (шаг 3) пройдено. Замечание З-1 устранено по варианту A (контракт эквивалентности
> replay, §2.5; DOD этапа 4). Остальные замечания (`arch_review.md`, М-1 … М-7) отложены до
> кодирования (шаг 4).

## 1. Ключевые решения

- **Модель записи отделена от кодирования.** Каноническая in-memory модель — `Record` в ядре;
  конвертация в байты — за трейтами `RecordWriter`/`RecordReader`. Дефолтная реализация — JSON
  (`JsonRecordCodec`, читаемость важна для цели №2); подстановкой другой реализации трейтов можно
  перейти на бинарный формат (сеть, большие архивы) без изменения модели и точек записи.
- **Новый крейт не заводится.** Вся запись/воспроизведение ядра — это уже ответственность
  `ynwa-core` (там же `serde`/`serde_json` в зависимостях, [`ynwa-core/Cargo.toml`](ynwa-core/Cargo.toml:8)):
  `journal`, `record`, `codec`, `replay`. Футбольно-специфичная часть остаётся в `ynwa-football`
  (события, pin мяча, фабрика replay-мира).
- **Журнал живёт в ядре**: [`Game`](ynwa-core/src/game.rs:254) получает приёмник событий
  (`JournalSink`), в который системы пишут через `Game::record`; события таймстампятся аргументом
  `timestamp` систем. Реализация приёмника решает, собирать события в память или писать потоком.
- **Воспроизведение конфигурирует игру сокращённым набором систем.** Replay-мир собирается
  отдельной фабрикой (`create_football_replay_world`), а не через обычную
  [`create_football_world`](ynwa-football/src/lib.rs:237). В нём работают только «pin мяча в
  `Setup`», `ReplayDriver` (подача событий из журнала) и
  [`PhysicsSystem`](ynwa-core/src/systems/physics.rs:28); Lua и системы принятия решений
  ([`DecisionSystem`](ynwa-core/src/systems/decision/decision_system.rs:124),
  [`BallPossessionSystem`](ynwa-core/src/systems/ball_possession.rs:97),
  [`ActionSystem`](ynwa-core/src/systems/action.rs:79)) в мир не добавляются.
- **RNG записывается исходами**: владение и фактическая скорость мяча после удара.
- **Шаг фиксированный, записывается один раз.** Неравномерный шаг на практике не используется —
  симуляция шагается с фиксированным шагом. В запись кладётся одно значение `fixed_dt` и число
  шагов `total_steps`; это упрощает формат и сохраняет точное воспроизведение физики.
- **Минимальное решение — без снапшотов, версионирования, кроссплатформенной гарантии и интеграции
  в плеер.** Они вынесены в отдельные задачи (`task_4_serialization_snapshots`,
  `task_5_serialization_versions`, `task_6_serialization_crossplatform`,
  `task_7_player_integration`); сетевая трансляция исключена.

## 2. Новые типы и изменения `ynwa-core`

### 2.1 `journal.rs` (новый модуль)

```rust
pub struct JournalEntry {
    pub timestamp: f32, // абсолютное время шага (аргумент System::update)
    pub event: JournalEvent,
}

pub enum JournalEvent {
    DecisionAssigned {
        player_index: usize,
        decision: Decision,
        reason: Option<String>,
    },
    PossessionChange {
        possessed_by: Option<usize>,
        last_possessing_team: Option<Team>,
    },
    KickOutcome {
        player_index: usize,
        ball_velocity: Velocity3D,
    },
    StageChange { stage: GameStage },
    RestartSet {
        restart_position: Option<Point3D>,
        restart_team: Option<Team>,
    },
    DecisionsReset,
    StatUpdate { team: Team, key: String, delta: f64 },
    /// Эскейп для спорт-специфичных семантических событий (футбол пишет сюда FootballEvent).
    External { kind: String, data: serde_json::Value },
}

/// Приёмник событий журнала; внедряется в Game (по аналогии с RngManager).
pub trait JournalSink {
    fn push(&mut self, timestamp: f32, event: JournalEvent);
}

/// No-op приёмник по умолчанию — обычная игра без записи ничего не пишет.
pub struct NullJournalSink;

impl JournalSink for NullJournalSink {
    fn push(&mut self, _timestamp: f32, _event: JournalEvent) {}
}
```

- Порядок событий задаётся `timestamp` и порядком добавления.
- `External` не интерпретируется ядром; его раскодирует `ynwa-football`
  (`decode_football_events`) для цели №2.

### 2.2 Изменения `Game` и `World`

- В [`Game`](ynwa-core/src/game.rs:254) добавляется поле `journal_sink: Box<dyn JournalSink>`
  (по умолчанию `NullJournalSink` — сигнатуры [`Game::new`](ynwa-core/src/game.rs:261) и
  [`Game::with_stage`](ynwa-core/src/game.rs:265) не меняются).
- Новые методы:
  - `Game::record(&mut self, timestamp: f32, event: JournalEvent)` — делегирует в
    `journal_sink.push(timestamp, event)`;
  - `Game::set_journal_sink(&mut self, sink: Box<dyn JournalSink>)` — записывающая система
    подключает себя перед стартом.
- [`World::step`](ynwa-core/src/world.rs:23) не меняется.

### 2.3 Точки записи в системах ядра

- [`DecisionSystem`](ynwa-core/src/systems/decision/decision_system.rs:124): записывает
  `DecisionAssigned` в трёх местах присвоения `current_decision`: (1) авто-`Stop` по прибытии,
  (2) успешное решение `DecisionMaker`, (3) решение из обработчика ошибки. Записывается решение
  уже в канонической (display) ориентации.
- [`BallPossessionSystem`](ynwa-core/src/systems/ball_possession.rs:97): при смене владения
  записывает `PossessionChange { possessed_by, last_possessing_team }`.
- [`ActionSystem`](ynwa-core/src/systems/action.rs:79): в ветке `Kick` после расчёта разброса
  записывает `KickOutcome { player_index, ball_velocity }` — фактическую скорость мяча.

### 2.4 Serde для доменных типов

- Кастомные `Serialize`/`Deserialize` для uom-обёрток ([`Point3D`](ynwa-core/src/field/zones.rs:11),
  [`Velocity3D`](ynwa-core/src/field/zones.rs:22)): сериализуются как `{x, y, z}` в метрах /
  м/с; `Angle` — в градусах, `Length` — в метрах.
- Derive для `Region`, `GridCell`, `Team`, `GameStage`, `Decision`, `DecisionTarget`, `PlayerDef`,
  `BallDef`, `RefereeDef`, `ScriptingConfig`, `GameConfig`, `JournalEntry`, `JournalEvent`.
- Кастомная сериализация с **детерминированным порядком** для коллекций: зоны
  [`Field`](ynwa-core/src/field/mod.rs:55) (сортировка по `(name, team)`, запись как `Vec<Zone>`),
  `PlayerDef.regions` (сортировка ключей), `set_piece_roles` (сортировка).
- Serde для `GameState`/`PlayerState`/`BallState`/`RefereeState`/`StatSet` в task_3 **не делается**:
  эти типы сериализуются только в снапшотах (task_4).

### 2.5 `replay.rs` (новый модуль) — `ReplayDriver`

`ReplayDriver` реализует [`System`](ynwa-core/src/system.rs:2), хранит курсор
`next_index: usize` (стартует с 0). В `update(game, timestamp)` применяет в порядке журнала все
записи с `timestamp <= timestamp`, начиная с курсора.

Таблица эффектов (в порядке применения):

| Событие | Действие при replay |
|---|---|
| `DecisionAssigned` | `current_decision = Some(decision)`, `decision_reason = reason`, `decision_processed = false`, `needs_decision = false`, `last_decision_time = timestamp`, `last_error = None`; затем действие как в `ActionSystem`: `Stop` → скорость 0; `Run(t)` → скорость через `calculate_velocity`; `Kick` → скорость игрока не меняется. В конце `decision_processed = true`. |
| `PossessionChange` | `ball_state.possessed_by`, `ball_state.last_possessing_team`, `last_possession_change_time = timestamp`; всем игрокам `needs_decision = true` (для точности состояния; на физику не влияет). |
| `KickOutcome` | `ball_state.velocity = ball_velocity`, `possessed_by = None`, `last_possession_change_time = timestamp`. |
| `StageChange` | `state.stage = stage`. |
| `RestartSet` | `state.restart_position`, `state.restart_team`. |
| `DecisionsReset` | всем игрокам: `current_decision = None`, `is_ready = false`, `needs_decision = true`. |
| `StatUpdate` | `state.team_stats[team].increment(key, delta)`. |
| `External` | ядром игнорируется. |

**Контракт эквивалентности replay (вариант A).** Replay воспроизводит только *физически
значимое* состояние. Сравнению подлежат: позиции и скорости игроков, позиция и скорость мяча,
`possessed_by`, `last_possessing_team`, `last_possession_change_time`, `stage`, `team_stats`,
`restart_position`, `restart_team`, `elapsed_time`. Координационные флаги игроков
(`needs_decision`, `is_ready`, `decision_reason`, `last_error`, `last_decision_time`,
`decision_processed`) **не входят в контракт replay**: их значения в replay-мире не гарантируются
и не сравниваются. Записи этих полей в таблице выше оставлены только для внутренней
согласованности `ReplayDriver`; на физику они не влияют (в replay-мире нет систем, которые их
читают).

## 3. Изменения `ynwa-football`

- Добавить зависимости `serde`, `serde_json`; derive `Serialize`/`Deserialize` для
  [`FootballEvent`](ynwa-football/src/events.rs:8).
- [`FootballGameManager`](ynwa-football/src/game_manager.rs:41):
  - в ветке `Play` перед `handle_event` записывать
    `External { kind: "football_event", data: serde_json::to_value(event) }`;
  - в `handle_event` записывать эффекты: `StageChange`, `RestartSet`, `DecisionsReset`,
    `StatUpdate` (счёт `score`);
  - в `assign_setup_decisions` записывать `DecisionAssigned` для каждого выданного решения
    (включая `Run` исполнителя и остальных игроков);
  - переход `Setup → Play` записывать как `StageChange(Play)`.
- Новый компонент `ReplaySetupBallPlacer`: в replay-мире идёт первой системой и повторяет фиксацию
  мяча из [`FootballGameManager`](ynwa-football/src/game_manager.rs:52) в `Setup`:
  `ball.position = restart_position.unwrap_or(initial_position)`, `velocity = 0`,
  `possessed_by = None`, `last_possessing_team = None`. Проверка выполняется в начале каждого
  `update` по текущей стадии — это воспроизводит «мяч фиксируется со следующего тика после
  перехода в `Setup`» (на тике перехода мяч ещё движется, как в оригинале).
- `create_football_replay_world(record) -> Result<World, String>` — фабрика replay-мира с
  сокращённым набором систем (`ReplaySetupBallPlacer` → `ReplayDriver` → `PhysicsSystem`); обычная
  [`create_football_world`](ynwa-football/src/lib.rs:237) для replay не используется.
- `decode_football_events(&[JournalEntry]) -> Vec<(f32, FootballEvent)>` — раскодирование
  `External` в футбольные события для интеграционных тестов.

## 4. Модель записи и кодек

### 4.1 Типы (`record.rs` в `ynwa-core`)

```rust
pub struct RecordHeader {
    pub config: GameConfig,
    pub initial_stage: GameStage, // начальная стадия для реконструкции состояния
    pub fixed_dt: f32,            // фиксированный шаг симуляции, записывается один раз
}

/// Полная запись в памяти — используется для чтения/воспроизведения.
pub struct Record {
    pub header: RecordHeader,
    pub total_steps: u64,
    pub journal: Vec<JournalEntry>,
}
```

`Record` — кодек-независимая модель для чтения; запись ведётся потоково через `RecordWriter`
(§4.2).

- `Recorder` (в `record.rs`) — реализация `JournalSink` поверх `RecordWriter` + `Write`:
  создаётся с заголовком и сразу пишет его (`Recorder::new(header, writer)`); каждое событие из
  `push` пишет и сбрасывает в поток; `Recorder::finish(total_steps)` пишет трейлер. Драйвер хранит
  дескриптор `Recorder`, чтобы вызвать `finish` по завершении.

### 4.2 Кодек (`codec.rs` в `ynwa-core`)

```rust
/// Потоковая запись. Модель не зависит от формата.
pub trait RecordWriter {
    fn write_header(&mut self, header: &RecordHeader) -> Result<(), String>;
    fn write_event(&mut self, entry: &JournalEntry) -> Result<(), String>;
    fn finish(&mut self, total_steps: u64) -> Result<(), String>;
}

/// Чтение потока обратно в Record.
pub trait RecordReader {
    fn read(&mut self) -> Result<Record, String>;
}

/// Дефолтная реализация — JSON Lines (serde_json).
pub struct JsonRecordCodec;
```

- `JsonRecordCodec` — дефолт: реализует `RecordWriter`/`RecordReader` в формате JSON Lines
  (каждая строка — законченный JSON-объект). Читаемый, удобен для цели №2; при падении на диске
  остаётся валидный префикс.
- Бинарный кодек (сеть/архивы) — отдельная реализация `RecordWriter`/`RecordReader` (например,
  поверх `postcard` или `bincode`), добавляется позже; точки записи и модель `Record` не меняются.

### 4.3 Формат JSON (`JsonRecordCodec`)

Формат — JSON Lines (одна строка = один объект):

```jsonl
{"type":"header","config":{ "..." : "полный GameConfig" },"initial_stage":{"Setup":"kick off"},"fixed_dt":0.0166}
{"type":"event","timestamp":0.0166,"event":{"DecisionAssigned":{"player_index":0,"decision":{"Run":{"GridCell":{"col":13,"row":22}}},"reason":null}}}
{"type":"footer","total_steps":7200}
```

Каждая строка сбрасывается на диск сразу после записи — при падении файл остаётся читаемым
(заголовок + все записанные события) и воспроизводимым до последнего события.

### 4.4 API воспроизведения

- `create_football_replay_world(record: Record) -> Result<World, String>` (`ynwa-football`):
  строит `Game` через `Game::with_stage(record.header.config, record.header.initial_stage,
  dummy_rng)` и добавляет **сокращённый набор систем** в порядке:
  `ReplaySetupBallPlacer`, `ReplayDriver::new(0)`, `PhysicsSystem::new()`. Клиент (тест или плеер)
  выполняет `world.step(record.header.fixed_dt)` ровно `record.total_steps` раз.
- `decode_football_events(&[JournalEntry]) -> Vec<(f32, FootballEvent)>` (`ynwa-football`) —
  декодирование `External` обратно в `FootballEvent` для цели №2.

## 5. Детерминизм

- Сериализация детерминирована: итерации по `HashMap`/`HashSet` упорядочены (§2.4).
- Гарантия v1: детерминизм в рамках одной платформы (задокументировать в комментариях
  `ynwa-core`). Кроссплатформенный детерминизм — отдельная задача
  (`task_6_serialization_crossplatform`).

## 6. Начальное состояние

- Тесты: начальные позиции задаются через конфигурацию (регионы/стандартные ключи), как сейчас.
  Начальная стадия записывается в заголовок (`initial_stage`) и используется для реконструкции
  состояния при воспроизведении.

## 7. Порядок реализации

Реализация ведётся этапами; каждый этап завершается рабочим состоянием и минимум одним коммитом.

### Этап 1 — фундамент сериализации

- Кастомный serde для uom-обёрток (`Point3D`, `Velocity3D` и их `Length`/`Velocity`/`Angle`).
- Derive + детерминированный порядок для `Rectangle`/`Circle`/`Arc`/`PointZone`/`ZoneGeometry`/
  `Zone`/`Field`, `GridDimensions`/`GridCell`/`Region`, `Team`, `GameStage`, `Decision`/
  `DecisionTarget`, `PlayerDef`, `BallDef`, `RefereeDef`, `ScriptingConfig`, `GameConfig`.
- **DOD**: round-trip тесты `serialize → deserialize → equal` для перечисленных типов; поведение
  игры не меняется.

### Этап 2 — журнал и запись

- `journal.rs`: `JournalEvent`, `JournalSink`, `NullJournalSink`; `Game::record`/
  `Game::set_journal_sink`.
- Точки записи в `DecisionSystem`, `BallPossessionSystem`, `ActionSystem`, `FootballGameManager`.
- `Recorder` (в память) поверх `JournalSink`, собирающий `Record` (заголовок + события +
  `total_steps`).
- **DOD**: тест — прогнать короткую игру, собрать `Record`, убедиться, что в журнале есть
  ожидаемые события (решения, смена владения, удар, смена стадий).

### Этап 3 — кодек (JSON Lines)

- `record.rs`: `RecordHeader`/`Record`; `codec.rs`: `RecordWriter`/`RecordReader`,
  `JsonRecordCodec` (JSON Lines).
- **DOD**: round-trip `Record ↔ файл`; тест «обрыва» файла — читается валидный префикс
  (заголовок + события).

### Этап 4 — воспроизведение

- `replay.rs`: `ReplayDriver`; `ynwa-football`: `ReplaySetupBallPlacer`,
  `create_football_replay_world`, `decode_football_events`.
- **DOD**: интеграционный тест — исходный прогон и replay дают одинаковое физически значимое
  состояние (позиции и скорости игроков и мяча, владение, стадии, счёт, restart-поля,
  `elapsed_time`) и одинаковую последовательность `FootballEvent`; сравнение — по контракту
  эквивалентности из §2.5. Исходный прогон интеграционного теста запускается с детерминированной
  RNG-конфигурацией (`temperature = 0.0` или фиксированный `seed`), чтобы сам тест был
  воспроизводим.

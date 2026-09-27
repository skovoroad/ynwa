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

- **Заголовок**: полный [`GameConfig`](ynwa-core/src/game.rs:250) (поле, сетка, зоны, игроки,
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
[`Game`](ynwa-core/src/game.rs:276) (по аналогии с `rng_manager`) с методом записи; события
таймстампятся аргументом `timestamp`.

## 4. Случайность (RNG)

Предлагается писать в журнал **исходы** случайных веток, а не сырые значения генератора:
потребителей случайности в игре всего два — борьба за мяч
([`select_winner`](ynwa-core/src/systems/ball_possession.rs:43)) и разброс удара
([`ActionSystem::Kick`](ynwa-core/src/systems/action.rs:109)). Это делает воспроизведение
независимым от порядка розыгрышей и внутренностей RNG.

## 5. Детерминизм

- Для v1: гарантировать детерминизм в рамках одной платформы (задокументировать).
- Упорядочить итерации по `HashMap` (зоны — [`Field.zones()`](ynwa-core/src/field/mod.rs:154),
  статистика) для стабильного вывода сериализации; для логики с «первым совпадением» использовать
  детерминированный порядок.

## 6. Начальное состояние

- Для тестов достаточно начальных позиций с точностью до региона/стандартного ключа и начальных
  заданий — это выразимо уже сейчас через конфигурацию.

---

# Архитектурное решение (шаг 2)

> Предпосылка: **этап 1 (фундамент сериализации) выполнен** — serde для доменных типов
> (`Point3D`/`Velocity3D`, геометрия зон, [`Field`](ynwa-core/src/field/mod.rs:60) с
> детерминированным порядком зон, `Region`/`GridCell`, `Team`, `GameStage`, `Decision`,
> `PlayerDef`/`GameConfig`), базовые единицы СИ, round-trip тесты. Этап 1 не пересматривается;
> ниже — только оставшиеся этапы.

## 1. Ключевые решения

- **Модель записи отделена от кодирования.** Каноническая in-memory модель — `Record` в ядре;
  конвертация в байты — за трейтами `RecordWriter`/`RecordReader`. Дефолтная реализация — JSON
  Lines (`JsonRecordCodec`, читаемость важна для цели №2); подстановкой другой реализации трейтов
  можно перейти на бинарный формат (сеть, архивы) без изменения модели и точек записи.
- **`JournalSink` — чистый приёмник событий и шагов.** Трейт принимает события (`push`) и
  уведомления о завершении шага (`finish_step`), а также умеет финализироваться (`finish`);
  `Record` он не возвращает и не знает, «куда» идут события. Куда их девать — решает реализация:
  выбросить, сложить во внешнее хранилище, записать в файл, отправить по сети.
- **Шаги считает сам приёмник, а не клиент.** Игровой цикл ([`World::step`](ynwa-core/src/world.rs:23))
  на каждом шаге зовёт `finish_step`, поэтому приёмник ведёт счётчик шагов самостоятельно;
  `finish` не принимает `total_steps` — ни файловому (трейлер), ни коллекционному приёмнику внешний
  счётчик не нужен.
- **Накопление — во внешнем хранилище, а не внутри sink.** Во внешний `EventsCollection` (туда
  пишет `CollectJournalRecorder`) складываются события (`JournalEntry`) и число шагов прогона;
  `Record` в нём не хранится. `Record` собирается из коллекции и заголовка (§4.1).
- **Потоковая запись — отдельная реализация sink.** `FileJournalRecorder` сериализует каждое
  событие и сразу пишет в файл; в конце дописывает трейлер и сбрасывает буфер.
- **Новый крейт не заводится.** Вся запись/воспроизведение ядра — в `ynwa-core` (там же
  `serde`/`serde_json` в зависимостях): `journal`, `record`, `codec`, `replay`. Футбольно-специфичная
  часть остаётся в `ynwa-football` (события, pin мяча, фабрика replay-мира).
- **Журнал живёт в ядре**: [`Game`](ynwa-core/src/game.rs:276) владеет приёмником (`JournalSink`),
  системы пишут через `Game::record`; события таймстампятся аргументом `timestamp` систем.
- **Воспроизведение конфигурирует игру сокращённым набором систем.** Replay-мир собирается
  отдельной фабрикой (`create_football_replay_world`), а не через обычную
  [`create_football_world`](ynwa-football/src/lib.rs:237). В нём работают только «pin мяча в
  `Setup`», `ReplayDriver` (подача событий из журнала) и
  [`PhysicsSystem`](ynwa-core/src/systems/physics.rs:28); Lua и системы принятия решений
  ([`DecisionSystem`](ynwa-core/src/systems/decision/decision_system.rs:124),
  [`BallPossessionSystem`](ynwa-core/src/systems/ball_possession.rs:97),
  [`ActionSystem`](ynwa-core/src/systems/action.rs:79)) в мир не добавляются.
- **RNG записывается исходами**: владение мячом и фактическая скорость мяча после удара.
- **Шаг фиксированный, записывается один раз.** В запись кладётся одно значение `fixed_dt` и число
  шагов `total_steps` (его ведёт приёмник); это упрощает формат и сохраняет точное воспроизведение
  физики.
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
    /// Ядром не интерпретируется.
    External { kind: String, data: serde_json::Value },
}

/// Приёмник событий журнала. Куда девать события — решает реализация.
pub trait JournalSink {
    fn push(&mut self, timestamp: f32, event: JournalEvent);

    /// Уведомление о завершении шага; игровой цикл вызывает ровно один раз на шаг,
    /// поэтому приёмник сам ведёт счётчик шагов.
    fn finish_step(&mut self, timestamp: f32) {
        let _ = timestamp;
    }

    /// Финализация записи (трейлер, сброс буфера). Record не возвращает.
    fn finish(self: Box<Self>) -> Result<(), String> {
        Ok(())
    }
}

/// Приёмник по умолчанию — обычная игра без записи ничего не пишет.
pub struct NullJournalSink;

impl JournalSink for NullJournalSink {
    fn push(&mut self, _timestamp: f32, _event: JournalEvent) {}
}
```

- `finish` **не возвращает `Record` и не принимает `total_steps`**: число шагов приёмник считает
  сам по `finish_step`. Накапливающая реализация кладёт события и счётчик во внешнее хранилище
  (§2.2), потоковая — пишет трейлер с собственным счётчиком в файл (§4.3). `NullJournalSink`
  дефолтные `finish_step`/`finish` не переопределяет.
- Порядок событий задаётся `timestamp` и порядком добавления.
- `External` не интерпретируется ядром; его раскодирует `ynwa-football`
  (`decode_football_events`) для цели №2.

### 2.2 Внешнее хранилище событий и накапливающий приёмник

`EventsCollection` — хранилище событий и числа шагов прогона (не записей целиком): в него пишет
приёмник, из него читает вызывающий (в тестах — для проверок; при сборке `Record` — как источник
событий). `CollectJournalRecorder` — реализация `JournalSink`, направляющая события в общую
коллекцию и сама ничего не копящая.

```rust
#[derive(Default)]
pub struct EventsCollection {
    entries: Vec<JournalEntry>,
    total_steps: u64, // заполняется приёмником к моменту finish
}

impl EventsCollection {
    pub fn push(&mut self, entry: JournalEntry) { self.entries.push(entry); }
    pub fn entries(&self) -> &[JournalEntry] { &self.entries }
    pub fn total_steps(&self) -> u64 { self.total_steps }
    pub fn set_total_steps(&mut self, steps: u64) { self.total_steps = steps; }
    /// Забирает события и число шагов, дополняя их заголовком.
    pub fn take_record(&mut self, header: RecordHeader) -> Record {
        Record {
            header,
            total_steps: self.total_steps,
            journal: std::mem::take(&mut self.entries),
        }
    }
}

/// Sink поверх общей коллекции: сам ничего не копит.
pub struct CollectJournalRecorder {
    collection: Rc<RefCell<EventsCollection>>,
    steps: u64,
}

impl CollectJournalRecorder {
    pub fn new(collection: Rc<RefCell<EventsCollection>>) -> Self {
        Self { collection, steps: 0 }
    }
}

impl JournalSink for CollectJournalRecorder {
    fn push(&mut self, timestamp: f32, event: JournalEvent) {
        self.collection.borrow_mut().push(JournalEntry { timestamp, event });
    }

    fn finish_step(&mut self, _timestamp: f32) {
        self.steps += 1;
    }

    fn finish(self: Box<Self>) -> Result<(), String> {
        self.collection.borrow_mut().set_total_steps(self.steps);
        Ok(())
    }
}
```

- Коллекция общая: `Rc<RefCell<EventsCollection>>` держат и приёмник (внутри `Game`), и вызывающий.
  Это позволяет `Game` владеть `Box<dyn JournalSink>` без параметризации временем жизни.
- `total_steps` хранится в коллекции (её заполняет приёмник в `finish`), поэтому запись
  самодостаточна; заголовок знает вызывающий и объединяет его с коллекцией через `take_record`
  (§4.1).

### 2.3 Изменения `Game` и `World`

- В [`Game`](ynwa-core/src/game.rs:276) добавляется поле `journal_sink: Box<dyn JournalSink>`
  (по умолчанию `NullJournalSink` — сигнатуры [`Game::new`](ynwa-core/src/game.rs:283) и
  [`Game::with_stage`](ynwa-core/src/game.rs:287) не меняются).
- Новые методы:
  - `Game::record(&mut self, timestamp: f32, event: JournalEvent)` — делегирует в
    `journal_sink.push(timestamp, event)`;
  - `Game::finish_step(&mut self, timestamp: f32)` — делегирует в `journal_sink.finish_step`,
    уведомляя приёмник о завершении шага;
  - `Game::set_journal_sink(&mut self, sink: Box<dyn JournalSink>)` — подключает приёмник перед
    стартом;
  - `Game::finish_journal(&mut self) -> Result<(), String>` — отцепляет текущий sink, ставит
    `NullJournalSink` и вызывает `finish` (трейлер/flush). `Record` не возвращает.
- [`World::step`](ynwa-core/src/world.rs:23) в конце шага вызывает `Game::finish_step(new_timestamp)`
  — счётчик шагов ведёт сам приёмник, а не вызывающий.

### 2.4 Точки записи в системах ядра

- [`DecisionSystem`](ynwa-core/src/systems/decision/decision_system.rs:124): записывает
  `DecisionAssigned` (решение в канонической, display-ориентации) в трёх местах присвоения
  `current_decision`: (1) авто-`Stop` по прибытии, (2) успешное решение `DecisionMaker`,
  (3) решение из обработчика ошибки.
- [`BallPossessionSystem`](ynwa-core/src/systems/ball_possession.rs:97): при смене владения
  записывает `PossessionChange { possessed_by, last_possessing_team }`.
- [`ActionSystem`](ynwa-core/src/systems/action.rs:79): в ветке `Kick` после расчёта разброса
  записывает `KickOutcome { player_index, ball_velocity }` — фактическую скорость мяча.

### 2.5 Serde для моделей записи (продолжение этапа 1)

- Serde для `JournalEntry`/`JournalEvent` (нужен кодеку, §4) — добавляется на этапе 2.
- `GameState`/`PlayerState`/`BallState`/`RefereeState`/`StatSet` в task_3 **не сериализуются**:
  эти типы участвуют только в снапшотах (task_4).

### 2.6 `replay.rs` (новый модуль) — `ReplayDriver`

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

`ReplayDriver` переиспользует расчёт скорости из `ActionSystem`: `calculate_target_point` и
`calculate_velocity` выносятся в общий `pub(crate)` хелпер, чтобы физика replay совпадала с
оригиналом без дублирования.

**Контракт эквивалентности replay.** Replay воспроизводит только *физически значимое* состояние.
Сравнению подлежат: позиции и скорости игроков, позиция и скорость мяча, `possessed_by`,
`last_possessing_team`, `last_possession_change_time`, `stage`, `team_stats`, `restart_position`,
`restart_team`, `elapsed_time`. Координационные флаги игроков (`needs_decision`, `is_ready`,
`decision_reason`, `last_error`, `last_decision_time`, `decision_processed`) **не входят в контракт
replay**: их значения в replay-мире не гарантируются и не сравниваются. Записи этих полей в таблице
выше оставлены только для внутренней согласованности `ReplayDriver`; на физику они не влияют (в
replay-мире нет систем, которые их читают).

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
  - переход `Setup → Play` записывать как `StageChange(Play)`, конец игры — как
    `StageChange(GameOver)`.
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

## 4. Модель записи, кодек и потоковая запись

### 4.1 Типы (`record.rs`)

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

`Record` — кодек-независимая модель. Его **собирает вызывающий** из коллекции (§2.2), которая
сама знает и события, и число шагов:

```rust
let collection = Rc::new(RefCell::new(EventsCollection::default()));
game.set_journal_sink(Box::new(CollectJournalRecorder::new(Rc::clone(&collection))));

// прогон: world.step(fixed_dt) N раз; на каждом шаге World::step зовёт Game::finish_step

game.finish_journal()?; // приёмник переносит накопленное число шагов в коллекцию
let record = collection.borrow_mut().take_record(header);
```

### 4.2 Кодек (`codec.rs`)

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

- `JsonRecordCodec` — дефолт: `RecordWriter`/`RecordReader` в формате JSON Lines (каждая строка —
  законченный JSON-объект). Читаемый, удобен для цели №2; при падении на диске остаётся валидный
  префикс.
- Бинарный кодек (сеть/архивы) — отдельная реализация `RecordWriter`/`RecordReader` (например,
  поверх `postcard` или `bincode`), добавляется позже; точки записи и модель `Record` не меняются.

### 4.3 Потоковая запись — `FileJournalRecorder`

```rust
/// Sink поверх RecordWriter + Write: пишет каждое событие сразу, считает шаги,
/// копит первую ошибку записи.
pub struct FileJournalRecorder {
    writer: Box<dyn RecordWriter>,
    steps: u64,
    first_error: Option<String>,
}
```

- Конструктор пишет заголовок; `push` сериализует событие, пишет его и сбрасывает буфер, запоминая
  первую ошибку записи; `finish_step` инкрементирует счётчик шагов; `finish` пишет трейлер с этим
  счётчиком и возвращает первую накопленную ошибку (`Ok(())`, если ошибок не было).
- Ошибки записи всплывают единой точкой в `finish`: `Game::record`/`push` не возвращают `Result`,
  чтобы не менять сигнатуры систем.

### 4.4 Формат JSON (`JsonRecordCodec`)

Формат — JSON Lines (одна строка = один объект):

```jsonl
{"type":"header","config":{ "..." : "полный GameConfig" },"initial_stage":{"Setup":"kick off"},"fixed_dt":0.0166}
{"type":"event","timestamp":0.0166,"event":{"DecisionAssigned":{"player_index":0,"decision":{"Run":{"GridCell":{"col":13,"row":22}}},"reason":null}}}
{"type":"footer","total_steps":7200}
```

Каждая строка сбрасывается на диск сразу после записи — при падении файл остаётся читаемым
(заголовок + все записанные события) и воспроизводимым до последнего события.

### 4.5 API воспроизведения

- `create_football_replay_world(record: Record) -> Result<World, String>` (`ynwa-football`):
  строит `Game` через `Game::with_stage(record.header.config, record.header.initial_stage, dummy_rng)`
  и добавляет **сокращённый набор систем** в порядке: `ReplaySetupBallPlacer`, `ReplayDriver`,
  `PhysicsSystem`. Клиент (тест или плеер) выполняет `world.step(record.header.fixed_dt)` ровно
  `record.total_steps` раз.
- `decode_football_events(&[JournalEntry]) -> Vec<(f32, FootballEvent)>` (`ynwa-football`) —
  декодирование `External` обратно в `FootballEvent` для цели №2.

## 5. Детерминизм

- Сериализация детерминирована: итерации по `HashMap`/`HashSet` упорядочены (этап 1).
- Гарантия v1: детерминизм в рамках одной платформы (задокументировать в комментариях
  `ynwa-core`). Кроссплатформенный детерминизм — отдельная задача
  (`task_6_serialization_crossplatform`).

## 6. Начальное состояние

- Тесты: начальные позиции задаются через конфигурацию (регионы/стандартные ключи), как сейчас.
  Начальная стадия записывается в заголовок (`initial_stage`) и используется для реконструкции
  состояния при воспроизведении.

## 7. Порядок реализации

Реализация ведётся этапами; каждый этап завершается рабочим состоянием и минимум одним коммитом.
Этап 1 (фундамент сериализации) выполнен; ниже — оставшиеся этапы.

### Этап 2 — журнал и запись

- `journal.rs`: `JournalEvent`, `JournalEntry`, `JournalSink` (`push` + `finish_step` + `finish`),
  `NullJournalSink`; `EventsCollection`, `CollectJournalRecorder`.
- `record.rs`: `RecordHeader`, `Record`.
- [`Game`](ynwa-core/src/game.rs:276): поле `journal_sink`; методы `record`, `finish_step`,
  `set_journal_sink`, `finish_journal`; [`World::step`](ynwa-core/src/world.rs:23) зовёт
  `Game::finish_step` на каждом шаге.
- Точки записи в `DecisionSystem`, `BallPossessionSystem`, `ActionSystem`, `FootballGameManager`.
- **DOD**: тест — прогнать короткую игру с `CollectJournalRecorder`, убедиться, что во внешней
  коллекции есть ожидаемые события (решения, смена владения, удар, смены стадий) и правильное
  `total_steps`, и собрать из них `Record`.

### Этап 3 — кодек и потоковая запись (JSON Lines)

- `codec.rs`: `RecordWriter`/`RecordReader`, `JsonRecordWriter`/`JsonRecordReader` (JSON Lines);
  `FileJournalRecorder` поверх `RecordWriter` + `Write` (считает шаги через `finish_step`, пишет
  трейлер с этим счётчиком).
- `record_io.rs`: тонкие конструкторы `json_journal_file_writer`/`json_journal_file_reader` (файл) и
  `json_journal_memory_writer`/`json_journal_memory_reader` (память), скрывающие сборку `RecordWriter` в точке
  применения.
- **Согласовано с человеком:** тип `JsonRecordCodec` из архитектурного решения удалён как лишний
  уровень косвенности (его роль выполняют конструкторы конкретных типов и `record_io`). Файл
  `record_io.rs` архитектурным решением не описан, но ему не противоречит.
- **DOD**: round-trip `Record ↔ файл`; тест «обрыва» файла — читается валидный префикс
  (заголовок + события); тест потоковой записи — события попадают в файл по одному, не накапливаясь
  в памяти.

### Этап 4 — воспроизведение

- `replay.rs`: `ReplayDriver`; вынести `calculate_target_point`/`calculate_velocity` в общий
  `pub(crate)` хелпер.
- `ynwa-football`: `ReplaySetupBallPlacer`, `create_football_replay_world`, `decode_football_events`.
- **DOD**: интеграционный тест — исходный прогон и replay дают одинаковое физически значимое
  состояние (позиции и скорости игроков и мяча, владение, стадии, счёт, restart-поля,
  `elapsed_time`) и одинаковую последовательность `FootballEvent`; сравнение — по контракту
  эквивалентности из §2.6. Исходный прогон интеграционного теста запускается с детерминированной
  RNG-конфигурацией (`temperature = 0.0` или фиксированный `seed`), чтобы сам тест был
  воспроизводим.

### Этап 5 — пошаговое (lockstep) сравнение оригинала и replay

> Мотивация: интеграционные тесты этапа 4 сравнивают только **финальное** состояние и условия
> событий в моменты их детекции. Ошибки, которые самокорректируются к концу прогона или сдвигают
> применение события на шаг, такие тесты не ловят.

Ключевые решения:

- **Снапшоты по шагам — до и после шага.** Перед каждым `world.step` снимать `GameState`
  (реализует `Clone`) и результат [`check_events`](ynwa-football/src/events.rs:121) на нём; после
  шага снимать `GameState` ещё раз; replay прокрутить на то же число шагов с той же схемой.
  Сравниваются: контрактные поля по §2.6 на каждом снапшоте «до шага» и на финальном состоянии
  после последнего шага, а также `check_events` на каждом снапшоте «до шага». Снятие `check_events`
  **до** шага обязательно: [`FootballGameManager`](ynwa-football/src/game_manager.rs:79) вызывает
  `check_events` первой системой, то есть на состоянии конца предыдущего тика; событие,
  детектируемое на тике `T` (таймстамп журнала `T`), наблюдается именно на снапшоте до шага `T`.
  Снятие после шага сдвинуло бы последовательность на один тик относительно журнала.
- **Сравнивать только контрактные поля** (как в существующем `assert_equivalent`); координационные
  флаги игроков (`needs_decision`, `is_ready`, `last_error`, …) не сравниваются.
- **События обязаны присутствовать.** Тест утверждает, что за прогон произошло ожидаемое число
  событий заданных типов, иначе он вырождается в проверку чистой физики.

Как обеспечить события детерминированно:

- нулевой разброс RNG (`temperature = 0.0`) и скриптованный `DecisionMaker` вместо Lua;
- мяч ставится в стартовый регион игрока, чтобы владение и удар случались на первом тике `Play`;
- цель удара подбирается под событие: за боковую → `Touchline`, в чужие ворота → `Goal`, за линию
  ворот мимо створа → `GoalLine` (`corner`/`goal kick`);
- для **последовательности** событий в конфиг добавить игроку регионы стандартных положений
  ([`SET_PIECE_KEYS`](ynwa-football/src/lib.rs:125)) и роли `set_piece_roles`, иначе после первого
  события игра «залипает» в `Setup`;
- `GameEnd` — прогон полной длительности при `dt = 1.0`: ровно **121 шаг**. Причина:
  [`World::step`](ynwa-core/src/world.rs:23) сначала выполняет системы с `new_timestamp`, а
  `elapsed_time` присваивает только после них; поэтому [`check_game_end`](ynwa-football/src/events.rs:111)
  читает `elapsed_time` предыдущего шага, и `GameEnd` детектируется на шаге с таймстампом `121.0`
  (когда `elapsed_time` равен `120.0`). Начального `elapsed_time` в
  [`RecordHeader`](ynwa-core/src/record.rs:9) нет, поэтому стартовать часы с середины нельзя.

- **DOD**: lockstep-тест — запись многошагового прогона со сценарием, дающим несколько
  `FootballEvent` разных типов (минимум `Touchline` + `Goal`, при возможности `GoalLine`/`GameEnd`);
  на **каждом** шаге совпадают контрактные поля состояния и результат `check_events`;
  последовательность непустых `check_events` совпадает с декодированным журналом; тест утверждает
  наличие ожидаемых событий; `cargo fmt`, `cargo clippy`, `cargo test` без предупреждений; минимум
  один коммит.

---



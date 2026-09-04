# YNWA — контекст проекта

Документ восстанавливает контекст разработки: что за проект, как он устроен, из чего состоит каждый
модуль, какие контракты (Lua и JSON) действуют между слоями. Описан только продуктовый код;
тесты не описываются.

Актуальные источники правды: сам код и этот файл. Файлы `context_core.md` и `context_scripts.md` —
более ранние и местами расходятся с кодом (расхождения отмечены ниже в разделе «Известные
несоответствия»).

---

## 1. О проекте

Футбольный менеджер. Пользователь:

1. задаёт характеристики футболистов и их позиции (обычные и при стандартных положениях), а также
   пишет для них код принятия решений на Lua;
2. запускает матч и наблюдает за игрой.

Симуляция детерминирована по шагам (фиксированный timestep задаёт клиент), физика собственная,
ИИ игроков полностью вынесен в пользовательские Lua-скрипты.

### Крейты workspace

| Крейт | Роль |
|---|---|
| `ynwa-core` | Ядро симуляции: модель мира, системы, поле, регионы, ориентация, физика, исполнение решений. Не знает про футбол. |
| `ynwa-football` | Правила футбола: построение поля, стадии игры, события (гол, аут, конец матча), фабрика готового `World`. |
| `ynwa-decisions` | Независимая от игры библиотека исполнения Lua-скриптов. Общается только через JSON. |
| `ynwa-repository` | Файловая реализация `TeamRepository` из ядра (чтение `teams/`). |
| `ynwa-script-tests` | Инфраструктура интеграционного тестирования скриптов и визуальные сценарии. |
| `ynwa-player` | Локальный клиент (macroquad + egui). Не описывается. |
| `ynwa-scripts` | Библиотека Lua-преамбул (данные, без Rust). Контракты описаны в разделе 7. |
| `teams/` | Данные команд (TOML + Lua). Формат описан в разделе 8. |

Граф зависимостей (без циклов):

```
ynwa-player ──> ynwa-football ──> ynwa-core ──> ynwa-decisions
      │                                ▲
      └────────> ynwa-repository ──────┘  (реализует trait TeamRepository)
```

Ключевое архитектурное решение: `ynwa-core` не зависит ни от футбола, ни от способа хранения данных,
ни от клиента. Всё специфичное для вида спорта живёт в `ynwa-football`, всё специфичное для хранения —
в `ynwa-repository`.

---

## 2. Система координат и общие соглашения

- Правая система координат, Y вверх:
  - **X** — ширина поля,
  - **Y** — высота над газоном (физика 3D, отрисовка 2D сверху),
  - **Z** — длина поля; ворота команды A при Z ≈ 0, ворота команды B при Z ≈ length.
- «Display orientation» (каноническая) = перспектива команды A. Всё внутреннее состояние игры
  хранится только в ней.
- Перспектива команды B получается отражением: `x' = width - x`, `z' = length - z`, `y' = y`.
  Отражение применяется **только на границах системы принятия решений**:
  вход (context/GAME_DATA/регионы) отражается, выход (решение) отражается обратно.
  Благодаря этому скрипты обеих команд пишутся одинаково.
- Физические величины — `uom` (`Length`, `Velocity`, `Angle`), а не голые `f32`.
- Сетка поля: колонки — по X, ряды — по Z, нумерация 1-based, колонки в Excel-нотации
  (A=1 … Z=26, AA=27). Клетка — `"M22"`, регион — `"A1:B2"`.

---

## 3. `ynwa-core`

### Ответственность

Симуляция матча вне зависимости от вида спорта: модель сущностей, набор систем, поле и зоны,
адресация регионов, преобразования ориентации, физика и мост к Lua-решениям.

### Принятые решения

- **Гибридный ECS-подобный подход**: `config.players[i]` (неизменяемое) ↔ `state.player_states[i]`
  (изменяемое) — параллельные массивы, доступ O(1). Внешних ECS-библиотек нет.
- **Config / State разделены** явно: `GameConfig` не меняется во время матча.
- **Player/Ball/Referee — отдельные типы, не trait**: их обрабатывают разные системы, общий trait
  не дал бы выигрыша.
- **Poll-модель**: игровой цикл принадлежит клиенту, ядро предоставляет `World::step(delta_time)`.
- Системы получают `&mut Game` (а не `&mut World`) — иначе borrow checker не даст итерировать
  список систем; и получают **абсолютный timestamp**, а не delta — чтобы каждая система сама
  считала свои интервалы.

### Основные типы

#### `world.rs` — `World`
Владеет `Game` и упорядоченным списком `Box<dyn System>`. `step(delta_time)` вычисляет новый
timestamp, прогоняет все системы **в порядке добавления**, затем фиксирует `elapsed_time`.
Порядок систем — часть контракта (см. 3.1).

#### `system.rs` — `System`
Единственный метод `update(&mut self, game: &mut Game, timestamp: f32)`.

#### `game.rs`
- `Game` — контейнер `GameConfig` (приватный, доступ через `config()`) + публичный `state`.
  `Game::new` стартует со стадии по умолчанию `Setup("kick off")`; `Game::with_stage` задаёт стадию явно.
  При старте в `Setup` игроки ставятся **за пределами поля** в точку `(-5, 0, length/2)`; при старте
  в `Play`/`GameOver` — в центр своего региона `REGION_START_POSITION`.
- `GameConfig { field, players: Vec<PlayerDef>, ball: BallDef, referees, scripting }`.
- `GameState { elapsed_time, stage, player_states, ball_state, referee_states, team_stats,
  player_stats, restart_position, restart_team }`.
- `PlayerDef` — статические характеристики игрока (`reaction_rate`, `speed_rate`, `tackle_rate`,
  `shot_power`, `shot_accuracy`, все 10–100), его Lua-скрипт, карта именованных регионов
  `HashMap<String, Region>` и `set_piece_roles: HashSet<String>`. Ядро интерпретирует из регионов
  **только** ключ `REGION_START_POSITION = "start"` — это и есть контракт между ядром и
  спорт-специфичным слоем; `set_piece_roles` ядро не интерпретирует вовсе. Билдер-методы
  `with_*` задают характеристики.
- `PlayerState` — позиция, скорость, `current_decision`, `decision_reason`, флаги
  `needs_decision` / `decision_processed` / `is_ready`, `last_error`, `last_decision_time`.
- `BallState` — позиция, скорость, `possessed_by: Option<usize>`,
  `last_possessing_team: Option<Team>` (сохраняется во время передач),
  `last_possession_change_time` (для антидребезга).
- `Decision` — `Run(DecisionTarget)` | `Stop` | `Kick(Point3D)`.
  `DecisionTarget` — `Point` | `GridCell` | `Region` | `Ball`.
  Вариант `Ball` разрешается в актуальную позицию мяча: направление задаётся один раз в момент
  обработки решения (`ActionSystem`), а проверка «добежал» выполняется каждый тик по текущей позиции мяча.
- `GameStage` — `Play` | `Setup(String)` | `GameOver`. Строка в `Setup` — причина/тип
  стандартного положения (`"kick off"`, `"throw in"`, `"corner"`, `"goal kick"`). Само ядро смысла
  этих строк не знает.
- `ScriptingConfig` — тексты преамбул: core, stdlib, преамбулы команд A и B.
- `StatSet` — именованные `f64`-счётчики (`get`/`set`/`increment`). Ключи задаёт игровой слой
  (например, `"score"` пишет `FootballGameManager`). В Lua статистика **не** передаётся.

#### `team.rs` — `Team { A, B }` + `opposite()`.

#### `field/` — `Field`, `FieldBuilder`, `Zone`, `ZoneGeometry`
`Field` хранит ширину, длину, `GridDimensions` и зоны в `HashMap<(name, Option<Team>), Zone>` —
O(1) поиск по паре имя+команда (`get_zone("goal", Some(Team::A))`). Имя и команда продублированы
внутри `Zone`, чтобы зона была самодостаточной при передаче в рендер/логику.
`ZoneGeometry` — `Rectangle` | `Circle` | `Arc` | `Point`; примитивы валидируют вход в конструкторе
через `assert!`. `cell_size()` считает размер клетки как `width / columns` (клетки предполагаются
квадратными).

#### `region.rs` — `GridDimensions`, `GridCell`, `Region`, `RegionError`
Адресация прямоугольных участков поля.
- `GridCell::from_notation("AA10")`, `GridCell::column_to_label(col)`.
- `Region::from_grid_notation("A1:B2", dims)`, `to_grid_notation()` (одноклеточный регион
  печатается как `"M3"`).
- `Region::center(dims, field_width)` и `contains_point(...)` считают метрику из **квадратных**
  клеток размера `field_width / columns` (по обеим осям).
- Два способа создания — осознанное решение: `GridDimensions::create_region(...)` валидирует
  границы (для пользовательских данных), `Region::new(...)` не валидирует (для внутренне
  сгенерированных значений — результатов отражения, одноклеточных регионов).

#### `orientation.rs`
`flip_point_orientation`, `flip_grid_cell_orientation`, `flip_region_orientation`. При отражении
региона углы меняются местами, чтобы сохранить инвариант `top_left <= bottom_right`.

#### `physics_util.rs`
Модель скорости и удара:
- скорость игрока: `speed_rate / 100 * 10 м/с` (максимум ≈ 36 км/ч);
- `calculate_kick_velocity(shot_power, rng)` = `shot_power / 5` м/с с разбросом ±25 %;
- `calculate_kick_direction_with_accuracy(...)` — отклонение направления от ±5° при точности 100
  до ±45° при точности 10; `rng = 0.5` означает «без отклонения»;
- `distance`, `distance_2d` (без Y), `distance_length`.

#### `repository.rs` — `TeamRepository` и DTO
`trait TeamRepository { fn load_team(&self, team_id: &str) -> Result<TeamRecord, String> }`.
DTO: `TeamRecord { players, preamble }`, `PlayerRecord { static_data, tactical, script: Option<String> }`,
`PlayerStatic` (имя и 5 характеристик), `PlayerTactical { number, play_positions, set_piece_positions }`.
Ключи в обеих картах позиций — строки, их смысл задаёт спорт-слой; ядро их не трактует.

### 3.1 Системы (`systems/`) и порядок их выполнения

Порядок, который выставляет `ynwa-football::add_football_systems`, существенен:

1. **`FootballGameManager`** (из `ynwa-football`) — стадии и события.
2. **`PlayerReactionSystem`** — когда игрок готов принять новое решение.
   В `Play`: `needs_decision = true`, если прошёл интервал реакции
   (`0.5 c` при `reaction_rate=100` … `3.0 c` при `10`, линейно).
   В `Setup`: решение запрашивается только если у игрока его ещё нет, иначе флаг снимается
   (грубый фильтр; окончательный отсев делает `DecisionSystem`).
3. **`BallPossessionSystem`** — владение мячом. Полностью пропускается в `Setup`.
   Радиус борьбы 1 м, кулдаун смены владения 1 с (антидребезг). У мяча с владельцем отбирать может
   только соперник (партнёры не отбирают), свободный мяч могут забрать все в радиусе.
   Победитель выбирается вероятностно: `tackle_rate × random(0.5…1.5)`.
   Смена владения ставит `needs_decision = true` **всем** игрокам.
   Для тестов есть `with_rng()`.
4. **`DecisionSystem`** — координация принятия решений (см. ниже).
5. **`ActionSystem`** — превращает решение в скорость/удар.
6. **`PhysicsSystem`** — интегрирует скорость в позицию.

#### `DecisionSystem` (`systems/decision/decision_system.rs`)
Отделяет «когда решать» от «что решать»: стратегия внедряется через
`trait DecisionMaker { make_decision(&mut self, game, player_index) -> Result<(Decision, Option<String>), DecisionError> }`.
Реализации: `PlaceholderDecisionMaker` (заглушка — случайная клетка) и `ScriptedDecisionMaker` (Lua).

Важные нетривиальные моменты:
- **Проверка прибытия** выполняется каждый тик, независимо от стадии и таймера реакции: если игрок
  подошёл к цели своего `Run` ближе `0.5 м`, решение немедленно подменяется на `Stop`. Без этого
  игрок «проскакивал» бы цель, ожидая следующего вызова скрипта (до 3 секунд).
- В `Setup` скрипты **не вызываются**: решения раздаёт `FootballGameManager`. `DecisionSystem`
  в `Setup` только снимает `needs_decision` и делает проверку прибытия.
- Решение, полученное от `DecisionMaker`, приходит в перспективе команды и конвертируется в
  display-ориентацию функцией `convert_decision_to_display_orientation` (для команды A — no-op;
  `DecisionTarget::Ball` не отражается, так как разрешается в мировых координатах).
- Ошибка скрипта считается «состоявшейся попыткой» (`needs_decision = false`,
  `last_decision_time = timestamp`) — иначе получился бы шторм повторных вызовов падающего
  скрипта. Обработчик ошибок настраивается (`with_error_handler`), по умолчанию печатает в stderr
  и не выдаёт решения.
- `DecisionError` — `ScriptError` | `Timeout` | `RuntimeError`.

#### `ActionSystem` (`systems/action.rs`)
Обрабатывает каждое решение **ровно один раз** (флаг `decision_processed`).
- `Run` → вектор скорости к целевой точке с модулем `speed_rate/100 × 10 м/с`; если до цели
  меньше 0.5 м — скорость нулевая.
- `Stop` → нулевая скорость.
- `Kick` → работает, только если игрок владеет мячом: считает скорость и направление с учётом
  `shot_power`/`shot_accuracy`, затем **снимает владение** и сдвигает
  `last_possession_change_time`, чтобы `BallPossessionSystem` не вернул мяч бьющему на следующем тике.
  Если игрок мячом не владеет — решение молча игнорируется.

#### `PhysicsSystem` (`systems/physics.rs`)
Кинематика `position += velocity × dt`. Хранит собственное `last_update`, чтобы вычислить dt из
абсолютного timestamp. Мяч: если им владеют — позиция мяча приравнивается к позиции владельца и
скорость обнуляется; иначе мяч катится с трением (замедление 2 м/с²). Гравитации/отскоков нет.

#### `ScriptedDecisionMaker` (`systems/decision/scripted_decision_maker.rs`)
Адаптер между доменными типами ядра и JSON-API `ynwa-decisions`. Ответственность:
- `build_config()` — собирает JSON конфигурации движка: преамбулы команд и, для каждого игрока,
  его скрипт, команду и `static_data` (зоны поля и размеры поля/сетки), **уже отражённые** под
  перспективу его команды;
- `build_context()` — собирает JSON состояния на текущий тик (позиции, мяч, регионы игрока),
  тоже с отражением для команды B;
- делегирует парсинг в `decision_parser`, который **не** делает отражений (это делается на границе
  системы, в `DecisionSystem`).

Нетривиально:
- зоны-`Arc` при отражении получают пересчитанные углы (`180° − end`, `180° − start`);
- у региона в контексте есть `display_notation`: для команды A — обычная нотация (`"M3"`), для
  команды B — `"display (team)"`, например `"R42 (M3)"`, чтобы в логах/UI было видно и абсолютную,
  и «свою» координату;
- вызов `make_decision` в стадии `Setup` — это ошибка вызывающего кода, и метод намеренно падает
  с явным сообщением, а не тихо возвращает заглушку; в `GameOver` возвращается `Stop`.

#### `decision_parser.rs`
Преобразует JSON-решение в доменный `Decision` + `Option<reason>`. Границы сетки здесь не
проверяются (регион строится через `Region::new`) — валидность координат проявится позже.

#### `systems/decision/util.rs`
`resolve_target_point(decision, field_width, grid_dims, ball_state)` — единая точка разрешения цели
`Run` в `Point3D`; для `Ball` возвращает **текущую** позицию мяча (используется проверкой прибытия).

---

## 4. `ynwa-decisions`

### Ответственность

Игро-независимое исполнение Lua-скриптов принятия решений. Никаких доменных типов (нет `Point3D`,
`Region`, `Team`) — только `serde_json::Value` как lingua franca. Крейт можно опубликовать и
переиспользовать в другой игре.

### Принятые решения

- **JSON-контракт** вместо типизированного API: игровой движок и движок решений версионируются независимо.
- **Одна изолированная Lua VM на игрока**: скрипты не могут общаться через глобальные переменные.
- **Состояние между вызовами не сохраняется**: скрипт игрока перезагружается (`load().exec()`) на
  каждом вызове; сохраняются только преамбулы и глобали, выставленные при создании VM.
- **Песочница**: обнуляются `io`, `os`, `package`, `require`, `load`, `loadfile`, `dofile`,
  `debug`, `collectgarbage`, `_G`. Доступны `math`, `string`, `table` и базовые функции.
- **Таймаут** — опциональный hook раз в 10 000 инструкций (≈2× накладных расходов);
  `DecisionEngine` включает его жёстко на 100 мс.

### Типы

- **`DecisionEngine`** — фасад. Конструктор принимает JSON-конфиг и тексты core/stdlib преамбул,
  создаёт по `LuaExecutor` на игрока со склеенной преамбулой `core + stdlib + team` и записывает
  `GAME_DATA` из `static_data` игрока. `make_decision(player_index, context)` вызывает Lua-функцию
  `make_decision`, валидирует результат через `LuaDecision` и возвращает сырой JSON.
- **`LuaExecutor`** — низкоуровневая обёртка над `mlua`: песочница, преамбула, таймаут,
  `set_global`, `execute(script, fn, context)` и `execute_with_args(...)` (первый делегирует во
  второй с пустым списком аргументов). Контекст кладётся в глобальную переменную `context`.
- **`LuaDecision`** (`lua_format.rs`) — serde-схема допустимых решений (`Stop` / `Run` / `Kick`),
  используется только для валидации формата, без перевода в доменные типы.
- Ошибки: `ScriptError` (`SyntaxError`, `RuntimeError`, `SerializationError`,
  `DeserializationError`, `FunctionNotFound`, `Timeout`) и `DecisionEngineError`
  (`ScriptError`, `RuntimeError`, `Timeout`, `InvalidPlayerIndex`, `InvalidConfig`).

### JSON-протокол (контракт с `ynwa-core`)

**Конфигурация (при создании движка):**

```jsonc
{
  "team_preambles": { "team_a": "<lua>", "team_b": "<lua>" },
  "players": [
    {
      "script": "<lua или пустая строка>",
      "team": "team_a",              // ключ в team_preambles
      "static_data": {               // становится глобалью GAME_DATA в VM игрока
        "zones": { "goal_a": {...}, "penalty_area_b": {...}, "center_circle": {...} },
        "field": { "width": 68.0, "length": 104.6, "columns": 26, "rows": 40 }
      }
    }
  ]
}
```

Геометрия зоны — один из вариантов:
`{"type":"rectangle","min_x","max_x","min_z","max_z"}`,
`{"type":"circle","center_x","center_z","radius"}`,
`{"type":"arc","center_x","center_z","radius","start_angle","end_angle"}` (градусы),
`{"type":"point","x","z","tolerance"}`.
Имя зоны в JSON = `имя` + суффикс `_a`/`_b` для командных зон (нейтральные — без суффикса).

**Контекст (каждый вызов, глобаль `context`):**

```jsonc
{
  "me": {
    "team": "A",                 // "A" | "B"
    "number": 10,                // тактический номер в команде
    "index": 5,                  // глобальный индекс в массиве игроков (A: 0..10, B: 11..21)
    "position": {"x":..,"y":..,"z":..},
    "regions": {
      "start":   {"min_x":..,"max_x":..,"min_z":..,"max_z":.., "display_notation":"M3"},
      "attack":  {...}, "defence": {...}, "goal kick own": {...}, ...
    }
  },
  "teammates": [ {"index":0,"number":1,"position":{...}}, ... ],   // без самого игрока
  "opponents": [ {"index":11,"number":1,"position":{...}}, ... ],
  "ball": {
    "position": {"x":..,"y":..,"z":..},
    "owner_index": 5,            // null, если мяч свободен
    "owner_team": "A"            // "A" | "B" | "None"; сохраняется во время передач
  },
  "game": { "elapsed_time": 125.5 }
}
```

Все координаты и регионы уже приведены к перспективе команды игрока.

**Решение (возврат из Lua):**

```lua
{action = "stop"}
{action = "run", target_type = "point",  target = {x = .., z = .., y = 0}}
{action = "run", target_type = "cell",   target = "C7"}
{action = "run", target_type = "region", target = {from = "A5", to = "C7"}}
{action = "run", target_type = "ball"}                 -- поле target не нужно
{action = "kick", target = {x = .., z = .., y = 0}}
```

Любое решение может содержать необязательное строковое поле `reason` — оно сохраняется в
`player_state.decision_reason` и показывается в UI/логах. Это главный инструмент отладки поведения,
заполнять его следует всегда.

---

## 5. `ynwa-football`

### Ответственность

Правила футбола поверх ядра: геометрия поля, стадии матча и стандартные положения, детекция
событий, сборка готового `World`.

### Принятые решения

- Ядро не должно знать про футбол — поэтому весь этот слой вынесен в отдельный крейт; для другого
  вида спорта пишется аналогичный крейт.
- Создание поля и `GameConfig` — внутренние детали; наружу отдаётся только `World`.
- Крейт зависит от **trait** `TeamRepository`, а не от `ynwa-repository`: конкретную реализацию
  инжектит клиент.
- Все 16 ключей стандартных положений обязательны для каждого игрока и валидируются на старте:
  лучше упасть при загрузке, чем получить игрока без позиции посреди матча.

### `lib.rs` — точка входа

`create_football_world(repo: &dyn TeamRepository, preambles_path: &Path) -> Result<World, String>`:
строит поле, загружает `"team_a"` и `"team_b"`, валидирует наборы ключей и разметку `on_ball`,
конвертирует записи в `PlayerDef`, читает `core.lua`/`stdlib.lua`, создаёт `Game` в стадии
`Setup("kick off")` и добавляет системы в описанном выше порядке.

Существенные детали:
- `build_player_defs` парсит нотации позиций и **отражает регионы команды B** в абсолютные
  координаты поля (в TOML команда B пишет позиции в своей перспективе).
- Значение `"on_ball"` вместо нотации означает, что игрок — исполнитель этого стандарта; ключ
  попадает в `set_piece_roles`, а региона для него не создаётся.
- Ключ `"kick off opp"` дополнительно регистрируется как `REGION_START_POSITION` (`"start"`) —
  временное решение: там ждут все игроки до начала матча, пока не сделан выбор разыгрывающей
  команды.
- `SET_PIECE_KEYS` — 16 обязательных ключей; `ON_BALL_REQUIRED_KEYS` — 8 ключей `own`, в каждом из
  которых ровно один игрок команды обязан иметь `"on_ball"`; в ключах `opp` `"on_ball"` запрещён
  (мяч у соперника).
- Если `ScriptedDecisionMaker` не создался, система решений молча деградирует до
  `PlaceholderDecisionMaker` с предупреждением в stderr.
- Начальная позиция мяча берётся из зоны `center_spot`.

### `field_builder.rs`

`create_football_field()` — поле 68 × 104.615 м, сетка 26 × 40 (клетка ≈ 2.615 м).
`create_football_field_with_dimensions(...)` позволяет другие размеры, но **требует квадратных
клеток** (`width/columns == length/rows`), иначе возвращает ошибку — вся метрика регионов в ядре
считается от квадратной клетки.

Создаётся 19 зон: `field`, `half` (A/B), `goal_area` (A/B), `penalty_area` (A/B), `center_circle`,
`center_spot`, `penalty_arc` (A/B), `penalty_spot` (A/B), `corner_arc_bottom`/`corner_arc_top` (A/B),
`goal` (A/B). Ворота вынесены **за** линию поля (A: z ∈ [−2.5, 0], B: z ∈ [length, length+2.5]) —
именно так детектируется гол. Углы штрафной дуги вычисляются как пересечение окружности радиуса
9.15 м вокруг 11-метровой отметки с линией штрафной.

### `game_manager.rs` — `FootballGameManager`

Единственная система, знающая о стадиях.

- **`Setup(reason)`**: каждый тик мяч принудительно ставится в `restart_position` (или в начальную
  позицию мяча, если она не задана), скорость и владение обнуляются,
  `last_possessing_team` сбрасывается. Затем:
  - `assign_setup_decisions` — каждому игроку без решения выдаётся `Run`: исполнителю стандарта — в
    точку мяча, остальным — в их регион для соответствующего ключа. Если региона нет, пишется
    `last_error` и игрок остаётся на месте. Lua при этом **не вызывается**.
  - `check_player_readiness` — игрок считается готовым (`is_ready`), когда его текущее решение стало
    `Stop` (то есть `DecisionSystem` зафиксировала прибытие).
  - Когда готовы все — стадия переключается на `Play`.
- **`Play`**: вызывает `check_events` и обрабатывает первое найденное событие.
- **`GameOver`**: ничего не делает.

`handle_event`:
| Событие | Последствие |
|---|---|
| `Goal(team)` | `team` — владелец ворот, в которые забили, поэтому счёт увеличивается у `team.opposite()`. Переход в `Setup("kick off")`, `restart_position = None` (мяч в центре), `restart_team` = пропустившая команда. |
| `Touchline(pos, last)` | `Setup("throw in")`, мяч в точке пересечения, `restart_team = last.opposite()`. |
| `GoalLine(pos, last)` | Если за линию мяч вывела атакующая команда — `Setup("goal kick")`, мяч в 5.5 м от линии по центру, вводит защищающаяся команда; иначе — `Setup("corner")`, мяч в ближайшем угловом флаге, вводит атакующая команда. |
| `GameEnd` | `GameOver`. |

При любом переходе в `Setup` у всех игроков сбрасываются `is_ready`, `current_decision` и
поднимается `needs_decision`.

`resolve_set_piece_key(reason, player_team, restart_team, restart_position, w, l)` — маппинг
«причина + состояние» → один из 16 ключей. Логика различения:
- `own`/`opp` — совпадает ли `restart_team` с командой игрока;
- `left`/`right` — от лица игрока (левая сторона команды A — малые X, команды B — большие X);
- `own half`/`opp half` — в чьей половине мяч (A защищает `z < length/2`, B — `z > length/2`).
Неизвестная причина даёт `None`, что вызывающий трактует как ошибку.

Важное решение: `restart_position`/`restart_team` выставляются в `handle_event`, до начала тиков
`Setup`, — иначе их затёр бы сброс `last_possessing_team`, происходящий каждый тик `Setup`.

### `events.rs`

`FootballEvent` — `Goal(Team)` | `Touchline(Point3D, Team)` | `GoalLine(Point3D, Team)` | `GameEnd`.
`check_events` проверяет в порядке приоритета: гол → конец матча → аут по боковой → аут по лицевой.

Детали правил:
- гол: мяч **полностью** пересёк линию по Z (учитывается радиус 0.11 м), а по X учитывается только
  центр мяча — касание штанги изнутри считается голом, снаружи — нет;
- `check_goal_line` не срабатывает, если мяч между штангами (этот случай обслуживает `check_goal`);
- ауты требуют полного пересечения линии с учётом радиуса;
- команда, последней касавшаяся мяча, берётся из `last_possessing_team`, при `None` — `Team::A`
  (грубая заглушка);
- `GAME_DURATION = 120.0` секунд — временное значение для отладки, не полноценный матч.

---

## 6. `ynwa-repository`

### Ответственность

Файловая реализация `TeamRepository`: чтение команды из каталога.

### Принятые решения

- Изоляция способа хранения: заменив этот крейт на реализацию поверх БД, остальной код менять не
  нужно — ядро и футбольный слой знают только про trait.
- Каталоги игроков сортируются по имени — порядок загрузки детерминирован (`01`, `02`, …), и от
  него зависит глобальный индекс игрока.

### `FsTeamRepository`

`FsTeamRepository::new(base_path)`; `load_team(team_id)` читает `<base>/<team_id>/`:
- `preamble.lua` — обязателен (тактика команды);
- `players/NN/static.toml` и `players/NN/tactical.toml` — обязательны;
- `players/NN/script.lua` — необязателен (`None` ≡ пустой скрипт, игрок целиком играет по тактике
  команды);
- пустой список игроков — ошибка.
Промежуточные serde-структуры `StaticToml`/`TacticalToml` конвертируются в DTO ядра;
`meta.toml` (отображаемое имя команды) этим крейтом сейчас не читается.

---

## 7. Контракт Lua-скриптов

### Порядок загрузки

```
core.lua  +  stdlib.lua  +  preamble.lua команды  +  script.lua игрока
```

Преамбулы исполняются один раз при создании VM игрока, скрипт игрока перезагружается на каждом
вызове. `GAME_DATA` выставляется один раз, `context` — перед каждым вызовом.

### Модель диспетчеризации

`make_decision()` определён **в stdlib** и переопределять его нельзя. Он определяет состояние
владения и вызывает обработчик:

`player_play[state]` → `team_play[state]` → `error()`

Состояния: `"i_have_ball"`, `"ball_is_free"`, `"team_has_ball"`, `"opponent_has_ball"`.
Таблица `team_play` определяется в преамбуле команды, `player_play` — необязательное
переопределение в скрипте игрока (частичное: неуказанные состояния падают в `team_play`).
В стадии `Setup` Lua не вызывается вообще.

Тонкость: `"ball_is_free"` возникает только когда `owner_team == "None"`, то есть на старте и после
любого `Setup`. После передачи владение командой сохраняется, и состояние будет `"team_has_ball"`.

### Правила написания скриптов

1. Не обращаться к `context` и `GAME_DATA` напрямую вне `core.lua` — только через функции-обёртки;
   прямой доступ по сырым ключам (`GAME_DATA.zones.goal_a`) непереносим между командами.
2. Не переопределять `make_decision()`.
3. Не собирать таблицы `{action = ...}` вручную, если есть готовая функция stdlib.
4. Предпочитать point-free стиль: `team_has_ball = chase_ball`, а не обёртку в лямбду.

### `core.lua` — элементарный доступ к состоянию

`my_position()`, `my_index()`, `my_team_name()`, `my_regions()`, `ball_position()`, `ball_owner()`,
`get_ball_owner_team()`, `get_teammates()`, `get_own_goal()`, `get_opponent_goal()`,
`get_opponent_penalty_area()`. Функции с «own/opponent» скрывают привязку к команде — благодаря им
одна и та же тактика работает и за A, и за B.

### `stdlib.lua` — утилиты, действия и диспетчер

- Утилиты: `distance(p1,p2)` (2D), `parse_col(s)`, `parse_notation(n)`.
- Запросы состояния: `am_i_ball_owner()`, `is_in_region_obj(r)`, `is_in_region(from,to)`,
  `is_in_opponent_penalty_area()`, `get_teammate_by_number(n)`.
- Примитивные действия: `stop(reason)`, `chase_ball()`, `run_to_region_obj(r, reason)`,
  `run_to_region(from,to)`, `kick_to_cell(n)`, `kick_to_region(from,to)`,
  `kick_to_opponent_goal()`, `pass_to_teammate(tm)`, `pass_to_players_by_numbers(numbers)`.
- Тактические действия (используют именованные регионы игрока): `run_to_start_position()`,
  `run_to_attack_position()`, `run_to_defence_position()`, `run_to_opponent_penalty_area()`,
  `default_goalkeeper_cover_position()` (держит линию ворот по Z из региона `defence`, X следует за
  мячом с зажимом по ширине ворот).
- Диспетчер `make_decision()`.

Все функции-действия сами заполняют `reason`.

---

## 8. Формат данных команды (`teams/<team_id>/`)

```
meta.toml        # отображаемое имя команды
preamble.lua     # тактика команды: таблица team_play (и общие таблицы вроде goalkeeper_play)
players/NN/
  static.toml    # name, reaction_rate, speed_rate, tackle_rate, shot_power, shot_accuracy (10..100)
  tactical.toml  # number, [play_positions], [set_piece_positions]
  script.lua     # необязательно: player_play
```

`tactical.toml`:

```toml
number = 1

[play_positions]
attack  = "N9"
defence = "N1"

[set_piece_positions]
"kick off own"                = "N3"
"kick off opp"                = "N3"     # дополнительно регистрируется как "start"
"goal kick own"               = "on_ball"
"goal kick opp"               = "N5"
"corner own left"             = "N34"
"corner own right"            = "N34"
"corner opp left"             = "N3"
"corner opp right"            = "N3"
"throw in own left own half"  = "N3"
"throw in own left opp half"  = "N3"
"throw in own right own half" = "N3"
"throw in own right opp half" = "N3"
"throw in opp left own half"  = "N3"
"throw in opp left opp half"  = "N3"
"throw in opp right own half" = "N3"
"throw in opp right opp half" = "N3"
```

- Значение — нотация клетки/региона (`"K7"`, `"A1:B2"`) либо маркер `"on_ball"`.
- Все 16 ключей обязательны; `"on_ball"` допустим только в ключах `own` и ровно у одного игрока
  на ключ.
- Позиции пишутся в перспективе своей команды; для команды B движок отражает их при загрузке.
- `number` — тактический номер (1…N, без пропусков внутри команды). Игровые номера на футболках —
  отдельная будущая сущность.
- Ключи `[play_positions]` (`attack`, `defence`) используются только скриптами через `my_regions()`.

---

## 9. `ynwa-script-tests`

Инфраструктура проверки того, что Lua-скрипты через полный конвейер систем дают ожидаемые решения.

- `src/lib.rs` — хелперы построения мини-игр: игра с одним игроком и заданным скриптом, с
  преамбулами и без, на заданной стадии; загрузка фикстур; `request_decisions_for_all` для обхода
  таймингов `PlayerReactionSystem`.
- `fixtures/` — минимальные таблицы диспетчеризации и скрипт-шпион.
- `scenarios/<name>/teams/` — самодостаточные наборы команд для **визуальной** проверки конкретных
  игровых ситуаций; запуск: `./run_scenario.sh <scenario_name>` (передаёт клиенту каталог сценария и
  каталог преамбул).

---

## 10. Принципы разработки

**Код.** Типобезопасность (`uom` вместо `f32`), валидация в конструкторах через `assert!`,
описание данными вместо алгоритмов, идиоматичный Rust (`Option`/`Result`), O(1) на горячих путях,
YAGNI — реализуется только явно запрошенное. Регулярно прогонять `cargo clippy` и `cargo fmt`.

**Тесты.** Юнит-тесты в отдельных файлах `*_tests.rs` рядом с реализацией, подключаются через
`#[cfg(test)] #[path = "..."] mod tests;` — это уменьшает размер файлов реализации и контекст для
агентов. Тривиальные присваивания не тестируются; проверяются логика, границы, интеграция.

**Комментарии и документация.** Комментировать только неочевидное: «почему», а не «что».
Архитектурные решения — сюда, в контекст; из кода они не извлекаются.

---

## 11. Известные несоответствия и незавершённые места

Проверено по коду на момент написания — полезно знать, чтобы не доверять старым докам:

- **Размеры поля.** `context_scripts.md` упоминает 60 × 101.5 м и сетку 26 × 44 — это параметры
  тестовых полей. Продакшн-поле: 68 × 104.615 м, сетка 26 × 40 (`field_builder.rs`).
- **Стартовая расстановка в `Setup`.** Комментарий в `game_manager.rs` говорит `(width/2, 0, −5)`,
  фактически `Game::with_stage` ставит игроков в `(−5, 0, length/2)` — сбоку от поля.
- **`GAME_DURATION = 120 с`** при комментарии «1 minute» — отладочное значение.
- **`REGION_START_POSITION` = алиас `"kick off opp"`** — временное решение; выбор разыгрывающей
  команды на старте матча ещё не реализован.
- **Последнее касание мяча** определяется через `last_possessing_team` (владение), а не через
  реальное касание; при `None` подставляется `Team::A`.
- **Границы сетки для решений из Lua не валидируются** в парсере — некорректный регион всплывёт
  позже, при расчёте центра.
- **Судья** (`RefereeDef`/`RefereeState`) существует как заготовка, ни одна система его не двигает.
- Ссылки в старых доках на `ynwa-scripts/context.md` и `config.rs` неактуальны: таких файлов нет.

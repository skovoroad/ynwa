# YNWA — форматы данных и протоколы

Документ собирает в одном месте все «контракты на данных»: JSON-протокол между игровым ядром и
движком решений, формат хранения команды на диске и то, что именно читает репозиторий, а также
форматы файлов сценариев интеграционного тестирования (§4).
Общий контекст проекта — в `context.md`.

---

## 1. JSON-протокол `ynwa-core` ↔ `ynwa-decisions`

`ynwa-decisions` — игро-независимая библиотека: она не знает доменных типов ядра и общается с ним
только через `serde_json::Value`. Протокол состоит из трёх частей: конфигурация (один раз при
создании движка), контекст (на каждый вызов) и решение (возврат из Lua). Со стороны ядра протокол
реализует `ScriptedDecisionMaker` (`ynwa-core/src/systems/decision/scripted_decision_maker.rs`),
со стороны движка — `DecisionEngine` (`ynwa-decisions/src/decision_engine.rs`).

### 1.1 Конфигурация (при создании движка)

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

*Примечание:* поле `static_data` намеренно дублируется для каждого игрока. Это позволяет
трансформировать (отразить по осям X и Z) координаты зон индивидуально для каждой команды. Таким
образом, скрипты обеих команд видят поле со своей собственной «домашней» перспективы (где свои
ворота всегда сзади), сохраняя единый код.

Геометрия зоны — один из вариантов:
`{"type":"rectangle","min_x","max_x","min_z","max_z"}`,
`{"type":"circle","center_x","center_z","radius"}`,
`{"type":"arc","center_x","center_z","radius","start_angle","end_angle"}` (градусы),
`{"type":"point","x","z","tolerance"}`.
Имя зоны в JSON = `имя` + суффикс `_a`/`_b` для командных зон (нейтральные — без суффикса).

### 1.2 Контекст (каждый вызов, Lua-глобаль `context`)

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

Поле `display_notation` региона: для команды A — обычная нотация (`"M3"`), для команды B —
`"display (team)"`, например `"R42 (M3)"`, чтобы в логах/UI было видно и абсолютную, и «свою»
координату.

*Примечание по производительности:* хотя регионы и метаданные игрока стабильны во время матча,
они передаются на каждом тике в составе `context` для упрощения API со стороны Lua-скриптов.
В будущем это может быть оптимизировано (разделение на статический и динамический контексты),
если профилирование выявит проблемы с сериализацией.

### 1.3 Решение (возврат из Lua)

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

Формат валидируется в `ynwa-decisions` схемой `LuaDecision` (`lua_format.rs`), перевод в доменные
типы ядра делает `decision_parser.rs` в `ynwa-core`. Границы сетки в парсере не проверяются —
некорректный регион всплывёт позже, при расчёте центра.

---

## 2. Формат данных команды на диске (`teams/<team_id>/`)

Команда — это каталог с тактикой на Lua и подкаталогами игроков. Именно эту структуру читает
`FsTeamRepository` (см. §3). Каталоги игроков сортируются по имени, порядок задаёт глобальный
индекс игрока.

```
meta.toml        # отображаемое имя команды (сейчас репозиторием не читается)
preamble.lua     # тактика команды: таблица team_play (и общие таблицы вроде goalkeeper_play)
players/NN/
  static.toml    # name, reaction_rate, speed_rate, tackle_rate, shot_power, shot_accuracy (10..100)
  tactical.toml  # number, [play_positions], [set_piece_positions]
  script.lua     # необязательно: player_play
```

### `tactical.toml`

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

Правила:

- Значение — нотация клетки/региона (`"K7"`, `"A1:B2"`) либо маркер `"on_ball"`
  (игрок — исполнитель стандарта; см. `context_football.md`).
- Все 16 ключей стандартных положений обязательны; `"on_ball"` допустим только в ключах `own`
  и ровно у одного игрока на ключ. Валидация выполняется при запуске (`ynwa-football`).
- Позиции пишутся в перспективе своей команды; для команды B движок отражает их при загрузке.
- `number` — тактический номер (1…N, без пропусков внутри команды). Игровые номера на футболках —
  отдельная будущая сущность.
- Ключи `[play_positions]` (`attack`, `defence`) используются только скриптами через
  `my_regions()`.

---

## 3. Что читает `ynwa-repository`

`FsTeamRepository::load_team(team_id)` читает каталог `<base>/<team_id>/` и превращает его в DTO
ядра (`TeamRecord` / `PlayerRecord`, определены в `ynwa-core/src/repository.rs`):

- `preamble.lua` — обязателен → `TeamRecord.preamble`;
- `players/NN/static.toml` — обязателен → `PlayerStatic` (имя и 5 характеристик);
- `players/NN/tactical.toml` — обязателен → `PlayerTactical { number, play_positions,
  set_piece_positions }`; ключи обеих карт позиций — строки, их смысл задаёт спорт-слой;
- `players/NN/script.lua` — необязателен (`None` ≡ пустой скрипт, игрок целиком играет по тактике
  команды);
- пустой список игроков — ошибка;
- `meta.toml` сейчас не читается.

---

## 4. Форматы файлов сценариев интеграционного тестирования

Сценарий `ynwa-integration-testing` — каталог с тремя TOML-файлами и набором команд. Все файлы
парсятся serde-структурами с `deny_unknown_fields`: неизвестный ключ — ошибка загрузки. Кодировки
значений: `Point3D` — `{x, y, z}` в метрах, `Velocity3D` — `{x, y, z}` в м/с, `Team` — `"A"`/`"B"`.
Начальное и конечное состояния используют **один и тот же частичный формат**: отсутствующее поле
для входа означает «не задавать», для выхода — «не проверять».

Как собрать сценарий целиком — в [`integration_testing.md`](integration_testing.md).

### 4.1 `initial_state.toml` — частичное начальное состояние

```toml
stage = "Play"                  # необязательно: "Play" | "Setup" | "GameOver"; по умолчанию Setup("kick off")
setup_reason = "kick off"       # имеет смысл только при stage = "Setup" (или без stage)

[ball]                          # необязательно
position = { x = 34.0, y = 0.0, z = 99.0 }
velocity = { x = 0.0, y = 0.0, z = 20.0 }
possessed_by = { team = "A", number = 9 }   # или "none" — мяч свободен
last_possessing_team = "A"                  # или "none"

[setup]                         # имеет смысл только при stage = "Setup"
restart_position = { x = 34.0, y = 0.0, z = 5.5 }
restart_team = "A"              # или "none"

[[players]]                     # необязательный список точных позиций в метрах
team = "A"
number = 9
position = { x = 34.0, y = 0.0, z = 10.0 }
```

Правила:

- `stage` не указан — мир стартует в `Setup("kick off")`; `setup_reason` учитывается и при явном
  `stage = "Setup"`, и при отсутствии `stage`.
- при старте в `Setup` (`stage` отсутствует или `"Setup"`) задание `[ball]` — **ошибка загрузки**:
  первый тик `FootballGameManager` безусловно перезаписывает мяч. Управление мячом в `Setup` —
  только через `[setup]`.
- случайность не настраивается: раннер жёстко использует детерминированный RNG (`temperature = 0.0`,
  фиксированный `seed`).
- `[[players]]` переопределяет только начальную позицию `PlayerState.position`; тактические
  регионы, ключи `set_piece_positions` и роли исполнителя из `tactical.toml` не меняются и видны
  скриптам. Игрок адресуется `{team, number}` (номер — из `tactical.toml`); номер должен быть
  положительным. Игроки, не указанные в списке, получают фабричную позицию.

### 4.2 `scenario.toml` — план прогона `[run]`

```toml
[run]
dt = 0.1                # длина шага, обязательна, положительное конечное число

[[run.stop]]            # типизированный список; останавливает ЛЮБОЙ критерий
when = "stage"          # смена стадии на целевую
stage = "Setup"         # "Play" | "Setup" | "GameOver"
setup_reason = "kick off"   # необязательно: точная причина Setup

[[run.stop]]
when = "event"          # наступление события
event = "Goal"          # Goal | Touchline | GoalLine | GameEnd
team = "B"              # необязательно; у GameEnd команда запрещена

[[run.stop]]
when = "steps"          # лимит шагов — страховочный критерий
steps = 1000

# [[run.stop]]
# when = "time"         # лимит модельного времени — страховочный критерий
# time = 60.0
```

Типы критериев (`when`): `"stage"`, `"event"`, `"steps"`, `"time"`.

- `stage` — `stage` + необязательный `setup_reason`; `Setup` без `setup_reason` матчится с любой
  причиной.
- `event` — `event` + необязательный `team`; `GameEnd` не несёт команду, пара `GameEnd` + `team` —
  ошибка загрузки.
- `steps` — лимит шагов; `time` — лимит модельного времени.

Обязателен хотя бы один страховочный критерий `steps` или `time` (иначе ошибка загрузки) — защита
от зацикливания. Критерии проверяются после каждого шага в порядке объявления; первый сработавший
попадает в `StopReason`.

### 4.3 `scenario.toml` — ожидания `[expect]`

Каждый пункт опционален; проверяются только перечисленные поля.

```toml
[expect]
journal_match = "exact"     # "exact" (по умолчанию) | "subsequence"

[expect.stop]               # необязательно
when = "event"
event = "Goal"
team = "B"
steps = 15                  # необязательно: точное число шагов (для stage/event/time)

[[expect.journal]]          # необязательно; по одной записи на строку
type = "kick_outcome"
player = { team = "A", number = 1 }
at = 1.5                    # необязательно: абсолютный timestamp (с допуском)
```

`[expect.stop]` — каким критерием завершился прогон (`when` и поля те же, что у `[[run.stop]]`),
плюс необязательное точное число шагов `steps` для `stage`/`event`/`time`; для `when = "steps"`
значение `steps` одновременно является и лимитом, и ожидаемым числом шагов.

`[[expect.journal]]` — по одной ожидаемой записи журнала, дискриминатор `type`:

| `type` | Необязательные поля |
|---|---|
| `decision_assigned` | `player = {team, number}`, `decision`, `reason`, `at` |
| `possession_change` | `possessed_by = {team, number} \| "none"`, `last_possessing_team`, `at` |
| `kick_outcome` | `player = {team, number}`, `ball_velocity`, `at` |
| `stage_change` | `stage` (+ `setup_reason` для `Setup`), `at` |
| `restart_set` | `position`, `team`, `at` |
| `decisions_reset` | `at` |
| `stat_update` | `team`, `key`, `delta`, `at` |
| `football_event` | `event` (обязателен: `Goal`/`Touchline`/`GoalLine`/`GameEnd`), `team`, `at` |

`decision` в `decision_assigned` задаётся видом решения (`"Stop"` / `"Run"` / `"Kick"`); цель
решения не сравнивается. Игроки везде адресуются `{team, number}` (номер — `tactical.toml.number`);
раннер резолвит пару в глобальный индекс поиском по `config.players`, дубликат или отсутствие —
ошибка загрузки. Поля владения/команды (`possessed_by`, `last_possessing_team`, `restart_set.team`)
принимают `{team, number}` / `"A"` / `"B"` либо `"none"`.

`journal_match` задаёт способ сравнения `[[expect.journal]]`:

- `"exact"` (по умолчанию) — полное совпадение длины и порядка;
- `"subsequence"` — перечисленные записи встречаются в фактическом журнале в указанном порядке,
  прочие игнорируются.

Сравнение: типы и порядок — точно; целые, перечисления, строки — точно; float-поля (`Point3D`,
`Velocity3D`, `delta`) — с допуском `TOLERANCE` (`1e-4`); `timestamp` (`at`) — с допуском
`max(dt, 1e-4)`. `at` — абсолютный timestamp от старта прогона (прогон всегда стартует с нулевого
модельного времени).

### 4.4 `final_state.toml` — ожидаемое конечное состояние

Тот же частичный формат, что и `initial_state.toml`, но «**отсутствующее поле = не проверять**»;
присутствующее поле сравнивается (координаты — с допуском). Дополнительно задаётся счёт. Файл
опционален: если его нет, конечное состояние не проверяется.

```toml
stage = "Setup"                # необязательно
setup_reason = "kick off"      # необязательно; с stage = "Setup" или без
score = { A = 1, B = 0 }       # необязательно: счёт по командам

[ball]                         # необязательно
position = { x = 34.0, y = 0.0, z = 10.0 }
velocity = { x = 0.0, y = 0.0, z = 0.0 }
possessed_by = { team = "A", number = 9 }   # или "none"
last_possessing_team = "A"                  # или "none"

[setup]                        # необязательно
restart_position = { x = 34.0, y = 0.0, z = 5.5 }
restart_team = "B"             # или "none"

[[players]]                    # необязательно
team = "A"
number = 9
position = { x = 34.0, y = 0.0, z = 10.0 }
```

`stage = "Setup"` без `setup_reason` — проверяется только, что стадия `Setup` (любой причины); с
`setup_reason` — точная причина. `score` сверяется со значением `"score"` в `team_stats` каждой
команды (с допуском). Позиции игроков и мяча — с допуском `TOLERANCE`; скорость мяча — также с
допуском.

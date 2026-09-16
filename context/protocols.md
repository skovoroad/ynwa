# YNWA — форматы данных и протоколы

Документ собирает в одном месте все «контракты на данных»: JSON-протокол между игровым ядром и
движком решений, формат хранения команды на диске и то, что именно читает репозиторий.
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

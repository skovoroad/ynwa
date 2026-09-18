# Lua-скрипты: справочник игрока

Документация на этапе разработки: справочник по скриптовой части для тех, кто настраивает
игроков и команды. Полные форматы данных (`static.toml`, `tactical.toml`, `context`, `GAME_DATA`,
решение) — [`protocols.md`](protocols.md); общий контекст — [`context.md`](context.md);
внутреннее устройство скриптов — [`scripting_developer.md`](scripting_developer.md).

ВАЖНО: написание пользователем скриптов на Lua это промежуточное состояние. В будущем будет разработан специальный язык для пользователя, который будет транспилироваться в Lua. Поэтому на этапе разработки многие возможности пользовательских Lua-скриптов (например, прямой вызов функций core.lua) не ограничиваются: они не будут доступны пользователю в реальном языке программирования пользователя. Тем не менее следует соблюдать разделение уровней абстракции.

## 1. Поле, координаты и сетка

Кратко:

- Правая система координат: **X** — ширина поля, **Z** — длина поля; свои ворота всегда «сзади»
  (меньший Z), ворота соперника «впереди» (больший Z).
- Скрипт всегда видит поле **со стороны своей команды**; позиции в файлах пишутся в своей
  перспективе, для команды B движок отражает их автоматически. Поэтому один и тот же код работает
  за обе команды.
- Сетка: колонки — по X в Excel-нотации (`A`…`Z`, затем `AA`), ряды — по Z, нумерация 1-based.
  Клетка — `"M22"`, регион — `"A1:B2"`.

## 2. Характеристики и позиции игрока

**Статические характеристики** (`static.toml`):

| Поле | Диапазон |
|---|---|
| `name` | строка |
| `reaction_rate` | 10–100 |
| `speed_rate` | 10–100 |
| `tackle_rate` | 10–100 |
| `shot_power` | 10–100 |
| `shot_accuracy` | 10–100 |

**Тактические данные** (`tactical.toml`):

| Поле | Смысл |
|---|---|
| `number` | тактический номер (1…N) |
| `[play_positions]` | `attack`, `defence` — обычные позиции (нотация клетки/региона) |
| `[set_piece_positions]` | 16 ключей стандартных положений (см. ниже) |

**Стандартные позиции** — 16 обязательных ключей `[set_piece_positions]`:

```
"kick off own", "kick off opp",
"goal kick own", "goal kick opp",
"corner own left", "corner own right",
"corner opp left", "corner opp right",
"throw in own left own half", "throw in own left opp half",
"throw in own right own half", "throw in own right opp half",
"throw in opp left own half", "throw in opp left opp half",
"throw in opp right own half", "throw in opp right opp half"
```

Значение ключа — нотация клетки/региона либо маркер `"on_ball"` (этот игрок — исполнитель
стандарта; допустим только в ключах `own` и ровно у одного игрока на ключ). Ключ
`"kick off opp"` дополнительно регистрируется как регион `start`.

## 3. Доступные функции

### 3.1 Доступ к состоянию ([`core.lua`](../ynwa-scripts/preambles/core.lua))

| Функция | Возвращает |
|---|---|
| `ball_owner()` | глобальный индекс владельца мяча или `nil`, если мяч свободен |
| `my_position()` | таблица `{x, y, z}` — моя позиция |
| `my_regions()` | таблица `{имя = регион}` — мои именованные регионы |
| `ball_position()` | таблица `{x, y, z}` — позиция мяча |
| `get_teammates()` | массив партнёров `{index, number, position}` (без самого игрока) |
| `my_index()` | мой глобальный индекс |
| `my_team_name()` | `"A"` или `"B"` |
| `get_ball_owner_team()` | `"A"`, `"B"` или `"None"` |
| `get_opponent_goal()` | объект зоны ворот соперника |
| `get_own_goal()` | объект зоны своих ворот |
| `get_opponent_penalty_area()` | объект зоны штрафной соперника |

### 3.2 Утилиты ([`stdlib.lua`](../ynwa-scripts/preambles/stdlib.lua))

| Функция | Параметры | Возвращает |
|---|---|---|
| `distance(p1, p2)` | две позиции | расстояние по горизонтали (без Y) |
| `parse_col(s)` | строка колонки | номер колонки (`A`→1, `Z`→26, `AA`→27) |
| `parse_notation(n)` | строка `"M22"` | `col, row` |

### 3.3 Проверки условий ([`stdlib.lua`](../ynwa-scripts/preambles/stdlib.lua))

| Функция | Параметры | Возвращает |
|---|---|---|
| `am_i_ball_owner()` | — | `true`, если мяч у меня |
| `is_in_region_obj(region)` | регион | `true`, если я внутри региона |
| `is_in_region(from, to)` | нотации углов | `true`, если я внутри региона `from:to` |
| `is_in_opponent_penalty_area()` | — | `true`, если я в штрафной соперника |
| `get_teammate_by_number(n)` | тактический номер | партнёр `{index, number, position}` или `nil` |

### 3.4 Функции принятия решения ([`stdlib.lua`](../ynwa-scripts/preambles/stdlib.lua))

Примитивные действия:

| Функция | Параметры | Действие |
|---|---|---|
| `stop(reason)` | причина | остановиться на месте |
| `chase_ball()` | — | бежать за мячом |
| `run_to_region_obj(r, reason)` | регион, причина | бежать в центр региона-объекта |
| `run_to_region(from, to)` | нотации углов | бежать в центр региона |
| `kick_to_cell(notation)` | клетка | удар в центр клетки |
| `kick_to_region(from, to)` | нотации углов | удар в центр региона |
| `kick_to_opponent_goal()` | — | удар по воротам соперника |
| `pass_to_teammate(tm)` | партнёр | пас партнёру |
| `pass_to_players_by_numbers(numbers)` | массив номеров | пас ближайшему из перечисленных; если никого — удар по воротам |

Тактические действия (используют именованные регионы игрока):

| Функция | Параметры | Действие |
|---|---|---|
| `run_to_start_position()` | — | бежать в центр региона `start` |
| `run_to_attack_position()` | — | бежать в центр региона `attack` |
| `run_to_defence_position()` | — | бежать в центр региона `defence` |
| `run_to_opponent_penalty_area()` | — | бежать в центр штрафной соперника |
| `default_goalkeeper_cover_position()` | — | занять позицию на линии ворот: Z из региона `defence`, X следует за мячом с зажимом по ширине ворот |

## 4. Преамбула команды и `team_play`

[`preamble.lua`](../teams/team_a/preamble.lua) — тактика команды. Определяет таблицу `team_play`
(поведение по умолчанию для всех игроков) и общие таблицы (например, `goalkeeper_play`).

`team_play` — таблица, ключи которой — состояния владения мячом:

| Состояние | Смысл |
|---|---|
| `"i_have_ball"` | мяч у меня |
| `"ball_is_free"` | мяч ничей (`owner_team == "None"`) |
| `"team_has_ball"` | мяч у моей команды (не у меня) |
| `"opponent_has_ball"` | мяч у соперника |

Краткий пример:

```lua
team_play = {
    ball_is_free      = press_or_defend,
    team_has_ball     = press_or_attack,
    opponent_has_ball = press_or_defend,
}
```

Диспетчер `make_decision()` определён в stdlib и вызывает обработчик в порядке
`player_play[state] → team_play[state] → error()`.

## 5. Скрипт игрока и `player_play`

[`script.lua`](../teams/team_a/players/10/script.lua) — необязательный скрипт игрока. Определяет
таблицу `player_play` — частичное переопределение: указанные состояния заменяют `team_play`,
неуказанные падают в `team_play`.

Краткие примеры:

```lua
-- наследовать готовую общую таблицу
player_play = goalkeeper_play

-- или переопределить отдельные состояния
player_play = { i_have_ball = forward_with_ball }
```

## 6. Регионы и зоны поля

**Регион игрока** в `my_regions()` — объект `{min_x, max_x, min_z, max_z, display_notation}`.
Ключи: `start`, `attack`, `defence` и ключи стандартных положений из `[set_piece_positions]`.
`display_notation` — нотация для показа в UI/логах.

**Зоны поля** лежат в `GAME_DATA.zones`; обращаться к ним следует через функции
`get_own_goal()`, `get_opponent_goal()`, `get_opponent_penalty_area()`. Имя зоны = имя +
суффикс `_a`/`_b` для командных зон (нейтральные — без суффикса).

Геометрии зон:

- `rectangle`: `{type, min_x, max_x, min_z, max_z}`;
- `circle`: `{type, center_x, center_z, radius}`;
- `arc`: `{type, center_x, center_z, radius, start_angle, end_angle}` (градусы);
- `point`: `{type, x, z, tolerance}`.

19 зон поля: `field`, `half` (A/B), `goal_area` (A/B), `penalty_area` (A/B), `center_circle`,
`penalty_arc` (A/B), `corner_arc_bottom`/`corner_arc_top` (A/B), `center_spot`,
`penalty_spot` (A/B), `goal` (A/B).

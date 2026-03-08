# Задачи: Декларативное управление расстановкой при стандартных положениях

Ссылка на ТЗ: `docs/set_piece_positions.md`

Задачи упорядочены так, чтобы после каждой задачи (кроме отмеченных) проект собирался, игра запускалась, а существующие тесты проходили.

---

## Этап 0. Предварительный рефакторинг

Цель: устранить архитектурные нарушения в модели данных до начала основной работы.

### Задача 0.1. Заменить именованные set-piece поля на `HashMap` в `PlayerTactical` ✅

**Проблема:** `PlayerTactical` в `ynwa-core/src/repository.rs` содержит поля `goal_kick_own_position`, `corner_own_left` и т.д. — это football-специфичные знания в sport-agnostic слое ядра.

**Решение:** заменить все опциональные set-piece поля одним:

```rust
// ynwa-core/src/repository.rs
pub struct PlayerTactical {
    pub number: u32,
    pub start_position: String,
    pub attack_position: String,
    pub defence_position: String,
    /// Sport-specific set-piece positions. Keys and their meaning are defined by the sport layer.
    /// Value is either a grid notation string (e.g. "K7") or the special marker "on_ball".
    pub set_piece_positions: HashMap<String, String>,
}
```

Аналогично обновить `TacticalToml` в `ynwa-repository/src/fs_team_repository.rs` — поля `goal_kick_own_position` и прочие заменяются на секцию `[set_piece_positions]` в TOML.

**Затрагивает:**
- `ynwa-core/src/repository.rs` — структура `PlayerTactical`
- `ynwa-repository/src/fs_team_repository.rs` — структура `TacticalToml`, функция `load_player`
- `ynwa-football/src/lib.rs` — функция `build_player_defs`: чтение из `set_piece_positions` по строковым ключам вместо именованных полей
- все `tactical.toml` в `teams/` и `ynwa-script-tests/scenarios/`

Существующая логика (какие ключи читает `ynwa-football`) не меняется — меняется только способ хранения.

---

### Задача 0.2. Объединить `"start"` и `"after_goal"` в единый reason `"kick off"` ✅

**Проблема:** движок использует два разных строковых reason для семантически одинакового события — мяч в центр, расстановка одинаковая:
- `"start"` — при старте игры (`Game::new()`)
- `"after_goal"` — после гола (`FootballGameManager::handle_event`)

Это приводит к дублированию в данных (два ключа в `tactical.toml` с одинаковым значением) и в Lua (`team_setup = { start = f, after_goal = f }`).

**Решение:** заменить оба на единый reason `"kick off"`:
- В `Game::new()` / `GameStage::default()`: `Setup("kick off")`
- В `FootballGameManager` при голе: `Setup("kick off")`
- В `game_manager.rs` doc-comment: обновить описание reasons
- В `teams/*/preamble.lua`: `team_setup = { ["kick off"] = run_to_start_position }`
- В `ynwa-scripts/preambles/stdlib.lua`: обновить `default_get_setup_position` и комментарии
- В тестах: заменить `"start"` / `"after_goal"` на `"kick off"` везде, где они фигурируют как reason

**Затрагивает:**
- `ynwa-football/src/game_manager.rs` — замена строк `"start"` и `"after_goal"`
- `ynwa-core/src/game.rs` — `GameStage::default()` если задан там
- `teams/team_a/preamble.lua`, `teams/team_b/preamble.lua`
- `ynwa-scripts/preambles/stdlib.lua`
- `ynwa-script-tests/fixtures/dispatch_spy.lua`, `team_a.lua`, `team_b.lua`
- `ynwa-script-tests/tests/dispatch_tests.rs`, `stdlib_functions.rs`
- `ynwa-football/src/tests/game_manager_tests.rs`

---

### Задача 0.3. Перенести `start_position` в `set_piece_positions`; выделить `play_positions` ✅

**Проблема:** `start_position`, `attack_position`, `defence_position` — три разнородных поля с именованными строками. `start_position` семантически является стандартным положением (начальный удар) и должна лежать в `set_piece_positions` наравне с goal_kick и corner. `attack_position` и `defence_position` — игровые позиции, не связанные со стандартами, и логично объединить их в отдельный контейнер.

**Решение:**

```rust
// ynwa-core/src/repository.rs
pub struct PlayerTactical {
    pub number: u32,
    /// Позиции в фазе Play. Ключи: "attack", "defence". Определяются видом спорта.
    pub play_positions: HashMap<String, String>,
    /// Позиции при стандартных положениях. Значение — grid notation или "on_ball".
    pub set_piece_positions: HashMap<String, String>,
}
```

В TOML:
```toml
[play_positions]
attack  = "K7"
defence = "B3"

[set_piece_positions]
"kick off"      = "K7"
"goal kick own" = "K7"
# ...
```

Маркер `"on_ball"` допустим только в `set_piece_positions`, не в `play_positions`.

Ключи из `play_positions` поступают в `PlayerDef.regions` как есть (`"attack"`, `"defence"`).
Ключи из `set_piece_positions` тоже поступают как есть; специальный случай: `"kick off"` дополнительно регистрируется под константой `REGION_START_POSITION` (`"start"`) — контракт с ядром для начального размещения игрока.

**Затрагивает:**
- `ynwa-core/src/repository.rs` — структура `PlayerTactical`
- `ynwa-repository/src/fs_team_repository.rs` — структура `TacticalToml`, функция `load_player`
- `ynwa-football/src/lib.rs` — `build_player_defs`: читать `attack`/`defence` из `play_positions`; добавить `"kick off"` в `set_piece_positions`
- `ynwa-decisions` / `scripted_decision_maker.rs` — обновить сериализацию в Lua-контекст (было `context.me.attack_position` / `context.me.defence_position`)
- все `tactical.toml` в `teams/` и `ynwa-script-tests/scenarios/`
- `ynwa-repository/src/player_tests.rs` — обновить тесты

---

### ✅ Задача 0.4. Добавить `set_piece_roles: HashSet<String>` в `PlayerDef`

**Проблема:** нет способа представить в ядре, что игрок является исполнителем стандарта, не смешивая это с регионами позиционирования.

**Решение:** добавить в `PlayerDef` (ynwa-core/src/game.rs) отдельное поле:

```rust
pub struct PlayerDef {
    // ...существующие поля...
    pub regions: HashMap<String, Region>,
    /// Set-piece types for which this player is the designated taker (goes to the ball).
    /// Keys match the setup reason string (e.g. "goal kick own", "corner own left").
    /// Populated by the sport layer (e.g. ynwa-football); core treats this as opaque data.
    pub set_piece_roles: HashSet<String>,
}
```

`regions` и `set_piece_roles` семантически непересекающиеся: для каждого стандарта игрок либо имеет позицию в `regions`, либо является исполнителем в `set_piece_roles`.

**Затрагивает:**
- `ynwa-core/src/game.rs` — структура `PlayerDef`, конструктор `PlayerDef::new`
- `ynwa-football/src/lib.rs` — `build_player_defs`: при значении `"on_ball"` в `set_piece_positions` — добавлять ключ в `set_piece_roles` вместо `regions`
- тестовые хелперы в `ynwa-script-tests/src/lib.rs`, создающие `PlayerDef` напрямую — добавить пустой `HashSet` как дефолт

На данном этапе `set_piece_roles` заполняется, но нигде не используется — подготовка для этапа 2.

> Зависит от задачи 0.3: `"kick off"` теперь находится в `set_piece_positions`, поэтому `build_player_defs` обработает его в том же цикле, что и остальные стандарты.

---

## Этап 1. Расширение данных

### Задача 1.1. Добавить throw_in-ключи в `tactical.toml` и в обработку в `build_player_defs`

После рефакторинга 0.1 структура Rust для хранения set-piece позиций уже готова (`set_piece_positions: HashMap<String, String>`). Задача — добавить восемь throw_in-ключей в данные и в логику `ynwa-football`.

**Ключи** (используются как строки в TOML и в `ynwa-football`):
```
throw in own left own half
throw in own left opp half
throw in own right own half
throw in own right opp half
throw in opp left own half
throw in opp left opp half
throw in opp right own half
throw in opp right opp half
```

**Затрагивает:**
- `ynwa-football/src/lib.rs` — `build_player_defs`: добавить обработку throw_in-ключей из `set_piece_positions` (регион или `on_ball`) наравне с существующими goal_kick и corner ключами
- все `tactical.toml` в `teams/` — добавить throw_in-ключи в секцию `[set_piece_positions]`

---

### Задача 1.2. Добавить валидацию `on_ball` при загрузке команды

В `ynwa-football/src/lib.rs` добавить проверку: для каждого типа стандартного положения ровно один игрок команды имеет значение `"on_ball"` в `set_piece_positions`. Нарушение — ошибка загрузки с понятным сообщением.

**Затрагивает:** `ynwa-football/src/lib.rs` — новая функция валидации, вызываемая из `create_football_world` после сборки обеих команд.

---

### Задача 1.3. Сделать все set-piece ключи обязательными

Добавить в `FsTeamRepository` (`ynwa-repository/src/fs_team_repository.rs`) проверку при загрузке: все ожидаемые ключи в `[set_piece_positions]` должны присутствовать. Отсутствие ключа — ошибка загрузки.

Список обязательных ключей определяется в `ynwa-football` (константы или перечисление) и передаётся при валидации. После выполнения задачи 0.3 этот список включает в том числе `"kick off"`.

**Затрагивает данные:** все `tactical.toml` в `teams/` должны содержать полный набор ключей для всех стандартов, включая начальный удар.

> ⚠️ Тестовые сценарии в `ynwa-script-tests/scenarios/` на этом этапе могут быть сломаны — их обновление в задаче 2.2.

---

## Этап 2. Логика движка для `on_ball`

### Задача 2.1. Реализовать поведение Setup-стадии в `FootballGameManager`

В `FootballGameManager` (ветка `Setup` в методе `update`): для каждого игрока, у которого `needs_decision == true`, назначить решение напрямую, без обращения к Lua-скрипту:

- если текущий `reason` присутствует в `set_piece_roles` игрока → `Decision::Run` к `restart_position`
- иначе: взять регион из `regions` по ключу `reason` — он обязан присутствовать (гарантируется валидацией на загрузке, задача 1.3); отсутствие — записать ошибку в `PlayerState::last_error`, решение не назначать (игрок останется на месте), игра продолжается

После назначения решения выставить `needs_decision = false`, чтобы `DecisionSystem` не перезаписал его Lua-вызовом.

Все остальные игроки (те, у кого `needs_decision == false` или `current_decision == Stop`) не затрагиваются.

---

### Задача 2.2. Обновить тестовые сценарии в `ynwa-script-tests/scenarios/`

Заполнить `tactical.toml` всех игроков во всех сценариях (`goal_kick_teamA_left`, `goal_kick_teamA_right`, `goal_kick_teamB_left`, `goal_kick_teamB_right`) полным набором обязательных ключей в секции `[set_piece_positions]`. Исполнителю стандарта назначить `"on_ball"` в соответствующем ключе. Удалить `script.lua` у игроков, которые использовали `player_setup` только для стандартов.

> Тестовые сценарии не покрываются требованием консистентности — допустимо временное нерабочее состояние начиная с задачи 1.3.

---

## Этап 3. Удаление старой логики

### Задача 3.1. Удалить `default_goal_kick_setup` из `stdlib.lua`

Удалить функцию `default_goal_kick_setup()` из `ynwa-scripts/preambles/stdlib.lua`. Убедиться, что ни один активный скрипт её не использует.

---

### Задача 3.2. Удалить `team_setup` из командных преамбул

Удалить таблицы `team_setup` из `teams/team_a/preamble.lua` и `teams/team_b/preamble.lua`. Убедиться, что игра запускается корректно.

---

### Задача 3.3. Удалить диспетчер `get_setup_position` и связанную логику из `stdlib.lua`

Удалить из `stdlib.lua`:
- функцию `get_setup_position(reason)`
- функцию `default_get_setup_position(reason)`

Убедиться, что `run_to_start_position()` больше не делегирует к `default_get_setup_position`.

---

### Задача 3.4. Удалить `player_setup` из скриптовой системы

- В `DecisionEngine` / `scripted_decision_maker.rs`: убрать вызов `get_setup_position` в ветке Setup — Lua-скрипты больше не опрашиваются для стандартных положений.
- Убрать передачу `setup_reason` как аргумента в Lua.
- Удалить упоминания `player_setup` из документации (`ynwa-scripts/context.md`, `context.md`).

---

### Задача 3.5. Убрать `team_setup`/`player_setup` из тестов

Обновить тесты в `ynwa-script-tests/tests/dispatch_tests.rs` и `basic_scripts.rs`: удалить тесты, проверяющие `get_setup_position`, `team_setup`, `player_setup`. Убедиться, что оставшиеся тесты проходят.

---

### Задача 3.6. Удалить `run_to_restart_position` из `stdlib.lua`

Функция `run_to_restart_position()` становится ненужной — исполнитель стандарта теперь направляется движком. Удалить функцию из `stdlib.lua` и связанные с ней `get_restart_position()` и `is_my_team_restarting()`, если они больше не используются нигде.

---

## Этап 4. Документация

### Задача 4.1. Обновить `ynwa-scripts/context.md`

- Убрать описание `team_setup`, `player_setup`, `get_setup_position`, `default_get_setup_position`, `run_to_restart_position`, `is_my_team_restarting`, `get_restart_position`.
- Добавить описание новых полей `tactical.toml` для стандартных положений и семантики `on_ball`.
- Обновить раздел про Setup stage.

### Задача 4.2. Обновить `context.md` (корневой)

Обновить раздел **Implemented Components** и описание `PlayerTactical` / `TeamRepository` в соответствии с новой моделью данных.

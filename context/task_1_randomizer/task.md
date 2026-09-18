# Задача

в различных частях проекта у нас используются элементы случайности с помощью генератора случайных чисел. Они генерируют разброс разных игровых величин в соответствии с заданными характеристиками (например, точностью удара). Это нужно, чтобы игра не повторялась одинаково при каждом запуске с одинаковыми вводными

однако для будущих интеграционных тестов нам нужна обратная ситуация: абсолютно воспроизводимая ситуация

предложение: добавить класс, настройку или систему, которая отвечает за случайные величины в остальных системах. В тестах она конфигурируется с нулевой температурой, такой, что случайность всегда одна и та же, а в игре — с заданной температурой, которая позволяет регулировать разброс значений

# Notes

В ActionSystem, BallPossessionSystem у нас УЖЕ есть with_rng. Но это, похоже, не централизованная настройка.

# Шаг 2: Анализ требований и выработка архитектурного решения

## Анализ текущего состояния

В проекте случайность используется в трёх местах:

1. [`ActionSystem`](ynwa-core/src/systems/action.rs:75) — разброс мощности и точности удара
   - Имеет `rng: Option<Box<dyn Fn() -> f32 + Send>>`
   - Метод `with_rng()` доступен только в тестовых сборках (`#[cfg(test)]`)
   - Вызывает `get_random()` дважды при ударе (для мощности и точности)
   - Клиент знает формулу: `calculate_kick_velocity(shot_power, rng_value)`

2. [`BallPossessionSystem`](ynwa-core/src/systems/ball_possession.rs:35) — вероятный выбор владельца мяча
   - Имеет `rng: Option<Box<dyn Fn() -> f32 + Send>>`
   - Метод `with_rng()` доступен всегда
   - Вызывает `get_random()` один раз на кандидата при выборе победителя
   - Клиент знает формулу: `tackle_rate * (0.5 + rng_value)`

3. [`PlaceholderDecisionMaker`](ynwa-core/src/systems/decision/decision_system.rs:64) — случайный выбор ячейки
   - Использует `rand::rng()` напрямую — **не конфигурируется**

**Проблема**: текущий паттерн децентрализован — каждая система хранит свой RNG.
Нет единого менеджера. `PlaceholderDecisionMaker` вообще не поддерживает настройку RNG.
Клиентский код (системы) знает формулы рандомизации — это нарушает принцип единого центра управления.

## Требования

1. Централизованный менеджер случайности, инициализируемый один раз при создании `Game`
2. Возможность регулировки недетерминированности (температура 0.0–1.0): 0.0 = полная детерминированность, 1.0 = полный рандом
3. Все источники случайности должны быть конфигурируемы через единый менеджер (включая `PlaceholderDecisionMaker`)
4. Клиент не знает формулы рандомизации — он передаёт базовое значение и диапазон вариации, а менеджер возвращает рандомизированное
5. В тестах — воспроизводимая случайность: `DefaultRngManager` с сидом или `SequenceRngManager` для конкретных сценариев
6. В плеере — настраиваемая температура
7. Первый этап: хардкод температуры в коде (не в конфиге)
8. Трейтовый дизайн `RngManager` для поддержки будущих реализаций: `ReplayRngManager` (воспроизведение записанной последовательности), `RecordRngManager` (запись случайностей живой игры)
9. Простой API: `rng_manager.randomize(base, variation_pct)` — одна строка для рандомизации

## Архитектурное решение

### 1. Типы и их зависимости

Модуль: `ynwa-core/src/rng.rs`

**`RngConfig`** — неизменяемая конфигурация. Создаётся один раз при создании Game, не меняется во время матча. Содержит два поля: `temperature: f32` (0.0–1.0) и `seed: Option<u64>`. `seed = None` — энтропийный сид (плеер), `seed = Some(seed)` — воспроизводимый сид (тесты). Передаётся в конструктор `DefaultRngManager::new(rng_config)`. Не хранится в `GameConfig`.

Валидация: конструктор проверяет `assert!((0.0..=1.0).contains(&temperature))`.

**`RngManager`** — трейт, определяющий интерфейс получения случайных значений. Имеет три метода (см. раздел 4).

Конкретные реализации:

- `DefaultRngManager` — использует `StdRng`; сид берётся из `RngConfig::seed` (`Some` → `StdRng::seed_from_u64`, `None` → энтропийный сид). Применяет формулу температуры.
- `SequenceRngManager` — (будущая реализация) возвращает значения из заданной последовательности (зацикленная). Используется для тестов с конкретными сценариями.
- `ReplayRngManager` — (будущая реализация) возвращает значения из записанного массива. Используется для воспроизведения записанной игры.
- `RecordRngManager` — (будущая реализация) записывает все вызовы в буфер. Используется для записи случайностей живой игры.

**Зависимости**: `Game` владеет `RngManager` (через `Box<dyn RngManager>`). `RngConfig` передаётся в конструктор `DefaultRngManager`, а не хранится в `GameConfig`. Системы получают `&mut Game` и получают доступ к `RngManager` через `game.rng_manager()`: методы трейта принимают `&self`, а реализация обеспечивает внутреннюю мутабельность (`RefCell`). Это позволяет брать случайное значение и одновременно читать `game.config()` / `game.state`.

### 2. Формула рандомизации

```
random_value = 1.0 + temperature * ((rand - 0.5) * 2.0 * variation_pct)
output = base * random_value
```

где `rand` — случайное число в [0, 1], `variation_pct` — процент вариации (например, 0.25 для ±25%).

Пример для `base = 80`, `variation_pct = 0.25` (±25%):

- temperature = 0.0 → output = 80.0 (всегда)
- temperature = 1.0, rand = 0.0 → output = 80 * 0.875 = 70.0
- temperature = 1.0, rand = 0.5 → output = 80.0
- temperature = 1.0, rand = 1.0 → output = 80 * 1.125 = 90.0
- temperature = 0.7, rand = 0.0 → output = 80 * 0.9375 = 75.0
- temperature = 0.7, rand = 1.0 → output = 80 * 1.0625 = 85.0

### 3. Каналы случайности

Каналы для будущих систем (выносятся за рамки первого этапа, YAGNI):
- `action.kick_power` — мощность удара
- `action.kick_accuracy` — точность удара
- `possession` — владение мячом
- `decision` — принятие решений

На первом этапе каналы не поддерживаются — все вызовы идут через глобальный `RngManager`.

### 4. API RngManager

```rust
pub trait RngManager: Send {
    /// Возвращает сырое случайное значение в [0, 1]. При temperature=0 всегда возвращает 0.5.
    fn next(&self) -> f32;

    /// Возвращает рандомизированное базовое значение
    /// variation_pct = 0.25 означает ±25% от base
    fn randomize(&self, base: f32, variation_pct: f32) -> f32;

    /// Возвращает симметричное отклонение от нуля в диапазоне [-max, +max]
    /// Используется для углового отклонения удара
    fn randomize_range(&self, max: f32) -> f32;
}
```

Методы принимают `&self` (решение принято на ревью шага 4): реализация обеспечивает внутреннюю мутабельность, поэтому системы вызывают рандомизацию через `game.rng_manager()`, не требуя `&mut`-доступа к самому менеджеру.

**Сводная таблица методов:**

| Метод | Формула | Диапазон | Пример использования |
|-------|---------|----------|----------------------|
| `next()` | `rand` (при temp=0 → 0.5) | `[0, 1]` | Выбор ячейки (DecisionMaker) |
| `randomize(base, pct)` | `base * (1.0 + temp * ((rand - 0.5) * 2.0 * pct))` | `base * [1-pct, 1+pct]` | Сила удара, коэффициент борьбы |
| `randomize_range(max)` | `temp * (rand - 0.5) * 2.0 * max` | `[-max, +max]` | Угловое отклонение удара |

### 5. Изменения в существующих системах

**ActionSystem**:
- Удалить поле `rng: Option<...>` и метод `with_rng()`
- Физическую логику вынести в отдельные функции (без rng):
  - `kick_speed(shot_power) -> f32` — делит `shot_power` на `KICK_POWER_DIVISOR = 5.0`
  - `max_kick_deviation(shot_accuracy) -> Angle` — из `shot_accuracy` (100 → ±5°, 10 → ±45°), тип `uom::angle::degree`
  - `rotate_kick_direction(target, ball, deviation) -> (f32, f32)` — поворот нормализованного вектора на `deviation`
- Рандомизацию выполнять через `RngManager`:
  - `game.rng_manager().randomize(kick_speed(shot_power), 0.25)` — сила удара (±25% вариация)
  - `game.rng_manager().randomize_range(max_kick_deviation(shot_accuracy).get::<degree>())` — угловое отклонение
- Константы `KICK_POWER_VARIATION_MIN/MAX` удалить (заменяются `variation_pct = 0.25`)

**BallPossessionSystem**:
- Удалить поле `rng: Option<...>` и метод `with_rng()`
- Заменить `self.get_random()` на `game.rng_manager().randomize(tackle_rate as f32, 0.5)` (±50% вариация)

**PlaceholderDecisionMaker**:
- Заменить `rand::rng()` на `game.rng_manager().next()`: столбец и строка выбираются равномерно как `(next() * columns).floor() + 1` (и аналогично для `rows`) с зажимом в `1..=columns` / `1..=rows`
- Контракт `DecisionMaker` не меняется: `make_decision` продолжает получать `&Game`, так как методы `RngManager` принимают `&self`

**System trait**:
- Без изменений — системы получают `&mut Game` и вызывают `game.rng_manager_mut()`.

### 6. Интеграция в Game

Менеджер инжектируется в Game через конструктор. `RngConfig` не хранится в `GameConfig` (чтобы не нарушать `#[derive(Debug, Clone)]` и не создавать два источника правды), а передаётся в конструктор `DefaultRngManager`.

```rust
pub struct Game {
    config: GameConfig,
    state: GameState,
    rng_manager: Box<dyn RngManager>,
}

impl Game {
    pub fn new(config: GameConfig, rng_manager: Box<dyn RngManager>) -> Self {
        Self::with_stage(config, GameStage::default(), rng_manager)
    }

    pub fn with_stage(config: GameConfig, stage: GameStage, rng_manager: Box<dyn RngManager>) -> Self {
        // ... existing logic ...
        Self { config, state, rng_manager }
    }

    pub fn rng_manager(&self) -> &dyn RngManager {
        self.rng_manager.as_ref()
    }
}
```

### 7. Примеры использования

**Создание и конфигурирование (один универсальный пример):**

```rust
// В тестах: полная детерминированность (фиксированный сид)
let rng_config = RngConfig::new(0.0, Some(42));
let game_config = GameConfig::new(...);
let rng_manager = DefaultRngManager::new(rng_config);
let game = Game::new(game_config, Box::new(rng_manager));

// В плеере: настраиваемая температура, энтропийный сид
let rng_config = RngConfig::new(0.7, None);
let game_config = GameConfig::new(...);
let rng_manager = DefaultRngManager::new(rng_config);
let game = Game::new(game_config, Box::new(rng_manager));
```

**Использование RngManager в системе:**

```rust
// ActionSystem: рандомизация силы удара (±25%)
let kick_power = game.rng_manager().randomize(player.shot_power as f32, 0.25);

// ActionSystem: рандомизация углового отклонения (симметричное ±max_deviation)
let accuracy_deviation = game.rng_manager().randomize_range(max_deviation);

// BallPossessionSystem: рандомизация коэффициента борьбы (±50%)
let score = game.rng_manager().randomize(player.tackle_rate as f32, 0.5);

// PlaceholderDecisionSystem: рандомизация выбора
let rand_val = game.rng_manager().next();
```

**Пример для игрока с shot_power = 80, temperature = 0.7:**

```rust
// kick_power = 80 * (1.0 + 0.7 * ((rand - 0.5) * 2.0 * 0.25))
// rand = 0.0 → 80 * 0.9375 = 75.0
// rand = 0.5 → 80 * 1.0 = 80.0
// rand = 1.0 → 80 * 1.0625 = 85.0
```

**Примечание о детерминированном поведении**: при `temperature = 0` в `BallPossessionSystem::select_winner` все score становятся пропорциональны `tackle_rate`, и при равных `tackle_rate` побеждает кандидат с меньшим индексом (sort стабилен). Поведение детерминированное, но его стоит явно зафиксировать в документации метода.

**Воспроизводимая запись и воспроизведение (будущая реализация, эскиз):**

```rust
// Запись случайностей во время игры
let rng_config = RngConfig::new(0.7, None);
let rng_manager = DefaultRngManager::new(rng_config);
let game = Game::new(game_config, Box::new(rng_manager));
// ... игра идёт, все случайности записываются в буфер ...
let recorded = rng_manager.into_recorded_values();

// Воспроизведение записанной игры
let replay_manager = ReplayRngManager::new(recorded);
let game = Game::new(game_config, Box::new(replay_manager));
// ... все вызовы randomize() возвращают точно те же значения, что и при записи ...
```

## План изменений

1. Создать `ynwa-core/src/rng.rs` с `RngConfig`, `RngManager` (трейт), `DefaultRngManager`
2. Добавить `Box<dyn RngManager>` в `Game`, изменить `Game::new` и `Game::with_stage` так, чтобы они принимали `rng_manager`; `RngConfig` передаётся в конструктор `DefaultRngManager`, а не хранится в `GameConfig`
3. Обновить `ActionSystem`: убрать `rng`, использовать `game.rng_manager().randomize()`
4. Обновить `BallPossessionSystem`: убрать `rng`, использовать `game.rng_manager().randomize()`
5. Обновить `PlaceholderDecisionMaker`: использовать `game.rng_manager().next()` с равномерным маппингом в `1..=columns` / `1..=rows`
5a. Не требуется: контракт `DecisionMaker` остаётся `&Game`, так как `RngManager` использует внутреннюю мутабельность (`&self`)
6. Обновить все тесты: убрать `with_rng()`, настроить `RngConfig` при создании `Game`
7. Прогонять `cargo test`, `cargo clippy`, `cargo fmt` после каждого этапа
8. Обновить context/context.md (§3 «with_rng() для тестов», §physics_util.rs) — шаг 6

## Ревью архитектурного решения (шаг 3)

Ревью пройдено.

## Ревью кода (шаг 5)

Ревью пройдено, замечания исправлены сразу:

- `PlaceholderDecisionMaker` выбирал клетку через `randomize_range`, из-за чего диапазон удваивался, а превышения зажимались в последний столбец/строку; заменено на `next()` с равномерным маппингом, добавлен `test_placeholder_decision_maker_maps_raw_value_to_cell`.
- Текст задачи приведён в соответствие с реализацией: внутренняя мутабельность `RngManager`, `game.rng_manager()`, неизменный контракт `DecisionMaker`.
- `RngConfig` в тестовых хелперах создаётся через `RngConfig::new` (валидация температуры), а не литералом структуры.
- Документация `BallPossessionSystem` уточнена: множитель ∈ [0.5, 1.5] достигается только при temperature = 1.0.

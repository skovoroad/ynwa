# Задача

Интегрировать запись/воспроизведение в плеер (`ynwa-player`): CLI-режимы `--record` и `--replay`.

Возможности записи и воспроизведения реализуются и проверяются в ядре без плеера; здесь — только интеграция в `ynwa-player`.

# Требования

## CLI

- Режимы задаются флагами: `--record` (запись) и `--replay <файл>` (воспроизведение).
- Позиционные аргументы `teams_path` (по умолчанию `teams`) и `preambles_path` (по умолчанию
  `ynwa-scripts/preambles`) сохраняются для обычного режима и режима записи. В режиме `--replay`
  они не используются: запись самодостаточна (`RecordHeader.config` содержит поле, игроков,
  скрипты и преамбулы).
- Папка записи: по умолчанию домашний каталог пользователя + `.ynwa/games`; переопределяется
  флагом `--out <dir>` (имеет смысл только вместе с `--record`). Создаётся при необходимости.
- Имя файла: `game_<YYYYMMDD-HHMMSS>.jsonl` (локальное время); при совпадении имён добавляется
  числовой суффикс `_2`, `_3`, ….
- `--record` и `--replay` одновременно — ошибка: сообщение в лог и завершение с ненулевым кодом.
- Перемотки, рестарта и прочих возможностей нет.

## Режим записи

- После создания мира к `Game` подключается файловый журнал: `json_journal_file_writer` +
  `FileJournalRecorder` с заголовком `RecordHeader::from_game(game, fixed_dt)`, где `fixed_dt` —
  текущий шаг симуляции (при `rate = 60` это `1/60`).
- Запись завершается при переходе в `GameStage::GameOver`: вызывается `finish_journal()`.
- При досрочном закрытии окна журнал финализируется best-effort; файл остаётся корректным
  префиксом (JSON Lines читается до последней полной строки).

## Режим воспроизведения

- Файл читается через `json_journal_file_reader` в `Record`.
- Футбольные события декодируются заранее через `decode_football_events(&record.journal)`.
- Мир воспроизведения создаётся через `create_football_replay_world(record)`.
- Воспроизведение идёт в том же темпе, что и игра (реальный 60 fps), шагами `record.header.fixed_dt`;
  останавливается после `record.total_steps` шагов.
- Решения игроков не пересчитываются (Lua и системы решений в replay не запускаются): они
  отображаются в UI из `PlayerState.current_decision` так же, как в обычной игре.
- Статистика (сейчас только счёт) воспроизводится из журнала (`StatUpdate` → `team_stats`).
- События игры отображаются в специальной панели — списке последних N событий, который постоянно
  дополняется: событие добавляется, когда `elapsed_time` достигает его записанного timestamp.
  Состав: события уровня игры (футбол) — `Goal`, `Touchline`, `GoalLine`, `GameEnd`.

## Ошибки

- Ошибки работы с файлами (файл не найден, повреждён, ошибка создания/записи) — сообщение в лог
  и завершение с ненулевым кодом.
- Проверка версий формата записи не выполняется (вынесена в `task_5_serialization_versions`).

# Проверка

- Ручная: запуск `--record`, проигрыш матча до `GameOver`, затем `--replay <файл>` и визуальная
  сверка физики, таблицы решений, счёта и списка событий.
- Автоматические тесты для `ynwa-player` не добавляются (GUI); ядро уже покрыто автотестами
  replay-эквивалентности и декодирования событий.

---

# Архитектурное решение (шаг 2)

Интеграция сосредоточена в крейте `ynwa-player`. Ядро (`ynwa-core`) и футбольный слой
(`ynwa-football`) не меняются. Разбор аргументов выделяется в новый модуль `cli.rs`, панель
событий — в `ui.rs`, управление темпом — доработка `simulation.rs`, режимы и главный цикл —
`main.rs`.

## 1. CLI (`cli.rs`)

Новые типы и функция разбора:

```rust
pub enum Mode {
    Play,
    Record,
    Replay(PathBuf),
}

pub struct Cli {
    pub mode: Mode,
    pub teams_path: PathBuf,
    pub preambles_path: PathBuf,
    pub out_dir: Option<PathBuf>,
}

pub fn parse(args: &[String]) -> Result<Cli, String>;
```

Разбор ручной, без добавления `clap`: перебор `args`, позиционные аргументы 1–2 задают
`teams_path` и `preambles_path` (по умолчанию `teams` и `ynwa-scripts/preambles`), флаги:
`--record`, `--replay <файл>`, `--out <директория>`.

Правила:
- `--record` и `--replay` одновременно → ошибка (лог + ненулевой код выхода).
- `--replay` без значения, `--out` без значения, неизвестный флаг → ошибка.
- Более двух позиционных аргументов → ошибка.
- Повторное указание одного и того же флага → ошибка.
- `--out` без `--record` → ошибка (см. «Решения по неоднозначным местам»).
- В режиме `Replay` позиционные аргументы `teams_path`/`preambles_path` игнорируются.

## 2. Каталог и имя файла записи

- `fn games_dir(out: Option<&Path>) -> Result<PathBuf, String>`: при заданном `out` используется
  он, иначе `env_home::home_dir().ok_or_else(|| "home directory not found".to_string())?.join(".ynwa/games")`;
  каталог создаётся через `std::fs::create_dir_all`.
- `fn next_record_path(dir: &Path) -> Result<PathBuf, String>`: базовое имя
  `game_<timestamp>.jsonl`, где `<timestamp>` — локальное время в формате `%Y%m%d-%H%M%S`;
  при совпадении имени добавляется суффикс `_2`, `_3`, … до первого свободного.

Зависимости `ynwa-player`: добавляются `chrono` (локальное время) и `env_home` (домашний
каталог; уже присутствует в графе зависимостей workspace).

## 3. `SimulationControl` (`simulation.rs`)

Разделяются «темп реального времени» и «шаг симуляции»: replay идёт в реальном темпе 60 fps,
но шагает на `fixed_dt` из заголовка записи.

```rust
pub struct SimulationControl {
    rate: f32,              // шагов реального времени в секунду (темп)
    step_delta: f32,        // dt, передаваемый в World::step
    paused: bool,
    accumulator: f32,
    steps_done: u64,
    max_steps: Option<u64>, // лимит шагов (только replay)
}
```

- `new(rate)` — обычный режим и запись: `step_delta = 1.0 / rate`, `max_steps = None`.
- `for_replay(fixed_dt, total_steps)` — `rate = 1.0 / fixed_dt` (при `fixed_dt = 1/60` это 60 fps),
  `step_delta = fixed_dt`, `max_steps = Some(total_steps)`.
- `pace() = 1.0 / rate` — интервал накопления кадрового времени; `should_step()` истинно при
  `accumulator >= pace()` и (для replay) `steps_done < max_steps`; `consume_step()` вычитает
  `pace()` и инкрементирует `steps_done`.
- `increase_rate`/`decrease_rate` в режиме записи и replay не применяются (`handle_input`
  получает флаг `rate_locked`); Space (пауза) работает во всех режимах.

## 4. Режим записи

Инициализация в `main`:
1. `let mut simulation = SimulationControl::new(60.0);` → `let fixed_dt = simulation.step_delta();`
   (при rate 60 это `1/60`).
2. Создание мира как сейчас — `create_football_world(&repo, &preambles_path)`.
3. Подключение файлового журнала:

```rust
let path = next_record_path(&games_dir(cli.out_dir.as_deref())?)?;
let writer = json_journal_file_writer(&path)?;
let header = RecordHeader::from_game(world.game(), fixed_dt);
world.game_mut().set_journal_sink(Box::new(
    FileJournalRecorder::new(Box::new(writer), &header),
));
```

(`Box::new(writer)` приводится к `Box<dyn RecordWriter>`.)

Главный цикл:
- `handle_input(..., rate_locked = true)` — `+`/`-` игнорируются, пауза работает.
- Шаги: `while simulation.should_step() { world.step(simulation.step_delta()); simulation.consume_step(); }`.
- После каждого шага проверяется `world.game().state().stage == GameStage::GameOver`; при первом
  срабатывании вызывается `world.game_mut().finish_journal()` (ошибка → лог + ненулевой код),
  в лог выводится путь файла, цикл завершается (приложение закрывается).
- Раннее закрытие окна (Esc): best-effort `let _ = world.game_mut().finish_journal();` и выход;
  файл остаётся корректным JSON Lines-префиксом. Повторный `finish_journal` безопасен (после
  первого sink — `NullJournalSink`), отдельный флаг не требуется.

## 5. Режим воспроизведения

1. `let mut reader = json_journal_file_reader(&path)?;` → `let record = reader.read()?;`
2. `let fixed_dt = record.header.fixed_dt; let total_steps = record.total_steps;`
3. `let events = decode_football_events(&record.journal);` — `Vec<(f32, FootballEvent)>` в
   хронологическом порядке.
4. `let world = create_football_replay_world(record)?;`
5. `let mut simulation = SimulationControl::for_replay(fixed_dt, total_steps);`

Главный цикл:
- `simulation.accumulate(get_frame_time());` (темп 60 fps).
- `while simulation.should_step() { world.step(simulation.step_delta()); simulation.consume_step(); }`.
- После `total_steps` шагов `should_step()` возвращает `false` — симуляция замирает, окно
  остаётся до Esc.
- `handle_input(..., rate_locked = true)` — `+`/`-` игнорируются, пауза работает.

Отображение решений и счёта без доработок: `ReplayDriver` заполняет
`PlayerState.current_decision`/`decision_reason`/`last_decision_time` из `DecisionAssigned`,
а `StatUpdate` пополняет `team_stats`, поэтому существующие `draw_player_decisions_table` и
`draw_score` работают как в обычной игре.

## 6. Панель событий (`ui.rs`)

- Константа `EVENT_PANEL_SIZE` (по умолчанию `5`) — сколько последних событий показывается.
- Новая функция `draw_event_panel(x, y, events: &[(f32, FootballEvent)])` выводит последние
  `EVENT_PANEL_SIZE` записей с меткой времени и текстом:
  - `Goal(team)` → «Гол: команда {team.opposite()}` (в событии — владелец ворот, забивший —
    противоположная команда);
  - `Touchline(_, last_team)` → «Аут; последнее касание: {last_team}»;
  - `GoalLine(_, last_team)` → «Лицевая; последнее касание: {last_team}»;
  - `GameEnd` → «Конец матча».
- `Team` не реализует `Display`: для текста используется хелпер `team_label(team)` → «A»/«B».
- Событие добавляется в видимый список, когда `timestamp <= world.game().state().elapsed_time`;
  в `main` для этого хранится курсор по `events`, снимаемый на каждой отрисовке.
- `render_scene` и `draw_control_panel` получают дополнительный параметр
  `events: &[(f32, FootballEvent)]`; панель рисуется только в режиме replay, в остальных
  режимах передаётся пустой срез.

## 7. Ошибки и коды выхода

Все ошибки аргументов и работы с файлами: сообщение в лог (`eprintln!`) и
`std::process::exit(1)` (macroquad-`main` возвращает `()`, поэтому ненулевой код задаётся
явно). Проверка версии формата записи не выполняется (вынесена в `task_5_serialization_versions`).

## 8. Тесты

Автотесты для `ynwa-player` не добавляются (GUI); проверка ручная по разделу «Проверка».

## Решения по неоднозначным местам

1. Изменение скорости (`+`/`-`) в режиме записи и replay запрещено: в записи это сохраняет
   соответствие `fixed_dt` в заголовке фактическим шагам, в replay — темп 60 fps. Пауза
   разрешена во всех режимах.
2. Значение `N` в панели событий — константа `EVENT_PANEL_SIZE = 5`.
3. `--out` без `--record` — ошибка с ненулевым кодом выхода.
4. В режиме записи по достижении `GameOver` журнал финализируется, после чего приложение
   завершает работу.
5. В replay после достижения `max_steps` накопление кадрового времени прекращается
   (аккумулятор не растёт бесконечно) — косметика.

---

# Ревью архитектурного решения (шаг 3)

Ревью пройдено. Архитектурное решение согласовано с кодом `ynwa-core` и `ynwa-football`;
замечания незначительные и внесены в текст решения выше:

- правила CLI дополнены: ошибка при более чем двух позиционных аргументах и при повторении флага;
- `env_home::home_dir()` возвращает `Option<PathBuf>` — обработан через `ok_or_else(...)?`;
- `for_replay` выводит `rate` из `fixed_dt` (`rate = 1.0 / fixed_dt`);
- для текста событий добавлен хелпер `team_label(team)` («A»/«B»), т.к. `Team` не реализует `Display`;
- уточнено поведение аккумулятора replay после достижения `max_steps`.

# Ревью этапа 2 задачи task_8 (шаг 5)

## Предмет ревью

Этап 2 архитектурного решения: `ynwa-football` — `FootballWorldBuilder` + строгая сборка.

Ревьюируемый коммит: `d3c0775` «task 8 step 2: add strict world build» (ветка
`task_8_integration_testing`). Изменения:

- [`ynwa-football/src/lib.rs`](ynwa-football/src/lib.rs) — переработка `add_football_systems`,
  новый `FootballWorldBuilder`, `create_football_world` как мягкая обёртка, правка `create_test_world`;
- [`ynwa-football/src/tests/world_builder_tests.rs`](ynwa-football/src/tests/world_builder_tests.rs) — новый файл тестов;
- [`context/tasks/task_8_integration_testing/task.md`](context/tasks/task_8_integration_testing/task.md) — уточнение постановки
  (`add_football_systems(world, decision_system)` вместо строгого `Result`-варианта).

Ориентир — исправленная версия постановки задачи; остальные изменения
([`context/template.md`](context/template.md) и т.п.) вне области ревью.

## Что проверено

- Соответствие исправленной постановке этапа 2 (§5.3, §6, §Декомпозиция/этап 2), включая раздел «Строгий режим сборки».
- Полнота и корректность нового кода, отсутствие регрессий в существующем поведении.
- Состав и полнота тестов (покрытие ветвей).
- [`context/code_requirements.md`](context/code_requirements.md) (простота, YAGNI, комментарии, покрытие тестами).
- `cargo test -p ynwa-football` — 131 тест зелёный.
- `cargo clippy --workspace --all-targets` — без предупреждений.
- `cargo fmt --all -- --check` — без изменений.

## Вывод по соответствию заданию

Реализация соответствует постановке:

- `add_football_systems(world, decision_system)` — принимает готовый `DecisionSystem`, без `Result`,
  без `println!`/`eprintln!`; все вызовы обновлены (в т.ч. `create_test_world`).
- Создание `ScriptedDecisionMaker::new` перенесено в `build()`: строгий режим возвращает `Err`,
  мягкий (`with_placeholder_fallback`) подставляет `DecisionSystem::new()` с предупреждением.
- `FootballWorldBuilder::new(repo, preambles_path)` с `with_rng`, `with_stage`,
  `with_placeholder_fallback` и строгим `build() -> Result<World, String>`; логика перенесена из
  `create_football_world`.
- `create_football_world` — мягкая обёртка с температурой `0.7` и placeholder-фолбэком.
- Удалён шумный `println!` из пути сборки; YAGNI не нарушен, лишних изменений нет.

Ветви нового решения покрыты тестами: успешная строгая сборка, строгая ошибка движка решений
(битая преамбула), ошибка чтения преамбулы, мягкий фолбэк, применение запрошенной стадии, обёртка.

## Замечания

### 1. Неточный doc-комментарий `with_placeholder_fallback` (несущественное)

[`ynwa-football/src/lib.rs`](ynwa-football/src/lib.rs:251): «falls back to a decision system that
makes no decisions» — неверно: фолбэк — это [`DecisionSystem::new()`](ynwa-core/src/systems/decision/decision_system.rs:91),
внутри которого [`PlaceholderDecisionMaker`](ynwa-core/src/systems/decision/decision_system.rs:49),
а он как раз принимает решения (случайный `Run` в клетку). Формулировку следует поправить
(например, «falls back to the placeholder decision system»).

### 2. Неточный doc-комментарий `create_football_world` (несущественное)

[`ynwa-football/src/lib.rs`](ynwa-football/src/lib.rs:337): «... so the game always starts» —
вводит в заблуждение: обёртка по-прежнему возвращает `Err` при ошибке репозитория, тактик или
чтения преамбул; мягким является только откат движка решений. Формулировку следует уточнить.

### 3. Тесты `world_builder_tests.rs` молча пропускаются при отсутствии `../teams` (несущественное)

Все тесты начинаются с `let Some(repo) = real_repository() else { return; }`
([`world_builder_tests.rs`](ynwa-football/src/tests/world_builder_tests.rs:20)). При отсутствии
каталога `../teams` тесты зеленеют, не проверяя ни одной ветви. Стиль согласован с существующим
`test_create_football_world_from_repository`, но для новых тестов строгой сборки это снижает
ценность. Варианты: `expect`/`panic!` при отсутствии фикстуры либо локальный фиктивный репозиторий
вместо файловой системы.

### 4. Тест фолбэка не проверяет, что использован именно placeholder (несущественное)

[`placeholder_fallback_builds_on_broken_preamble`](ynwa-football/src/tests/world_builder_tests.rs:47)
проверяет только успех сборки и число игроков. Что ветка фолбэка действительно задействована,
косвенно следует из парного строгого теста, но явного утверждения нет. Можно усилить (например,
проверкой, что прогон шага не падает и/или что сборка не использует скриптовый движок).

### 5. DOR шага 5: нет явной отметки об апруве в task.md (наблюдение)

DOR шага 5 требует наличия в [`task.md`](context/tasks/task_8_integration_testing/task.md) отметки
об апруве решения. В файле явной отметки нет; из истории коммитов видны `task 8 - description
approved` и `task 8 fix after arch review`, то есть апрув, вероятно, зафиксирован коммитами шагов 1–3.
Требуется подтверждение человека.

## Итог

Существенных замечаний нет. Все найденные замечания несущественные (формулировки doc-комментариев
и усиление тестов). Предлагается исправить их сразу по согласованию с человеком; возврат к шагу 4
не требуется.

## Резолюция

Согласовано с человеком: несущественные замечания исправлены сразу.

- Замечание 1 — [`with_placeholder_fallback`](ynwa-football/src/lib.rs:251): формулировка исправлена
  на «placeholder decision system».
- Замечание 2 — [`create_football_world`](ynwa-football/src/lib.rs:333): убрано «game always starts»,
  уточнено, что ошибки репозитория/тактик/преамбул по-прежнему возвращаются как `Err`.
- Замечание 3 — [`world_builder_tests.rs`](ynwa-football/src/tests/world_builder_tests.rs:20):
  `real_repository()` теперь падает с явным сообщением при отсутствии фикстуры вместо тихого
  пропуска теста.
- Замечание 4 — тест фолбэка теперь явно проверяет, что тот же битый репозиторий валит строгую
  сборку, прежде чем собрать мир через фолбэк.
- Замечание 5 — отметка об апруве: принято со слов человека (апрув зафиксирован коммитами шагов 1–3).

# Ревью задачи task_8 (шаг 5 экстра: мутационное тестирование)

## Предмет проверки

Задача [`task_8_integration_testing`](context/tasks/task_8_integration_testing/task.md): интеграционное
тестирование цепочки ядро — стандартная библиотека — скрипты игрока. Ветка
`task_8_integration_testing` (`HEAD` = `7c7e9c9`), сравнение с `master` (merge-base `4fc9d3a`).

Проверяется тестовое покрытие нового кода методом мутационного тестирования: в реализацию
вносились временные правки, портящие проверяемое поведение, и проверялось, что соответствующие
тесты падают. Мутации вносились и откатывались вручную (без `git`), после проверки рабочее дерево
возвращено в чистое состояние (`git status` чист).

## Что проверено

Новый код по крейтам и покрывающие его тесты:

- [`ynwa-core/src/snapshot.rs`](ynwa-core/src/snapshot.rs) + [`Game::apply_snapshot`](ynwa-core/src/game.rs:409)
  и [`ynwa-core/src/tests/snapshot_tests.rs`](ynwa-core/src/tests/snapshot_tests.rs);
- [`ynwa-football/src/lib.rs`](ynwa-football/src/lib.rs:222) (`FootballWorldBuilder`) и
  [`world_builder_tests.rs`](ynwa-football/src/tests/world_builder_tests.rs);
- [`ynwa-integration-testing/src/dto.rs`](ynwa-integration-testing/src/dto.rs),
  [`criterion.rs`](ynwa-integration-testing/src/criterion.rs),
  [`loader.rs`](ynwa-integration-testing/src/loader.rs),
  [`runner.rs`](ynwa-integration-testing/src/runner.rs),
  [`scenario.rs`](ynwa-integration-testing/src/scenario.rs),
  [`compare.rs`](ynwa-integration-testing/src/compare.rs) и их `*_tests.rs`;
- harness [`tests/scenarios.rs`](ynwa-integration-testing/tests/scenarios.rs).

Инструменты: `cargo test -p ynwa-core`, `-p ynwa-football`, `-p ynwa-integration-testing`
(включая `--test scenarios`). Базовый прогон зелёный: 100 unit-тестов + 8 тестов harness
(и ранее — тесты core/football).

## Результаты мутаций

Мутации, которые были надёжно пойманы тестами (тест падал):

- `apply_snapshot` (`ynwa-core`): стадия, позиция/скорость/владение мяча, `last_possessing_team`,
  позиции игроков, `restart_team`, счёт, граница валидации индекса (`>=` → `>`), снятие проверки
  индексов владения/игрока — 8 мутаций, все пойманы.
- `FootballWorldBuilder`: ветка строгой сборки, флаг `with_placeholder_fallback`, `with_stage` —
  3 мутации, все пойманы.
- `dto.rs`: запрет мяча в Setup, требование страховочного критерия, валидация `dt`, валидация
  номера игрока, токен `"none"` — 5 мутаций, все пойманы.
- `criterion.rs`: границы `steps`/`time`, учёт причины Setup, учёт команды события,
  запрет команды у `GameEnd` — 5 мутаций, все пойманы.
- `loader.rs`: дубликат игрока, значение по умолчанию `kick off`, `"none"` → `None`, обработка
  отсутствующего `final_state.toml`, резолв `{team, number}` — 5 мутаций, все пойманы.
- `runner.rs`: учёт `player_state.last_error`, передача новых событий в критерии — 2 мутации пойманы.
- `scenario.rs`: формирование `passed`/`diff` — 2 мутации пойманы.
- harness `tests/scenarios.rs`: выборка по фильтру, агрегация падений — 2 мутации пойманы.
- `compare.rs`: журнал (exact/subsequence), допуск timestamp, допуск координат/скоростей,
  отображение решения, матчинг команды/стадии, пропуск блока `final_state` и др. — большинство
  мутаций поймано.

## Замечания

Все замечания относятся к [`ynwa-integration-testing/src/compare.rs`](ynwa-integration-testing/src/compare.rs)
и касаются отсутствия **отрицательных** тестов для ветвей сравнения: проверяется только
совпадающий случай, а случай несовпадения — нет. Мутация «функция всегда возвращает совпадение»
оставляет весь набор тестов зелёным, то есть регресс, из-за которого сценарий с неверным
ожиданием начнёт проходить, тестами не будет пойман.

### 1. Не покрыта ветвь несовпадения `field_eq` (StatUpdate `team`/`key`)

[`field_eq`](ynwa-integration-testing/src/compare.rs:431) используется при сверке
[`StatUpdate`](ynwa-integration-testing/src/compare.rs:197) по `team` и `key`. Подстановка
`expected.as_ref().is_none_or(...) → true` не ломает ни один тест: положительный
[`journal_restart_set_and_stat_update_match`](ynwa-integration-testing/src/tests/compare_tests.rs:494)
проходит, а теста со неверным `team`/`key` в `StatUpdate` нет. Нужен тест: ожидаемый `StatUpdate` с
чужой командой/ключом против фактического — должен давать diff.

### 2. Не покрыта ветвь несовпадения `delta_field_matches` (StatUpdate `delta`)

[`delta_field_matches`](ynwa-integration-testing/src/compare.rs:440): `→ true` не ломает тесты.
Проверяется только совпадение `delta` в пределах допуска
([`compare_tests.rs`](ynwa-integration-testing/src/tests/compare_tests.rs:502)), а выход за допуск —
нет. Нужен тест с `delta` вне `TOLERANCE`, ожидающий diff.

### 3. Не покрыта ветвь несовпадения `velocity_field_matches` (KickOutcome)

[`velocity_field_matches`](ynwa-integration-testing/src/compare.rs:444): `→ true` не ломает тесты.
Положительный тест [`kick_outcome_checks_velocity_with_tolerance`](ynwa-integration-testing/src/tests/compare_tests.rs:231)
проверяет только попадание в допуск; расхождение `ball_velocity` не проверяется. Нужен тест с
фактической скоростью вне допуска.

### 4. Не покрыта ветвь несовпадения `point_field_matches` (RestartSet `position`)

[`point_field_matches`](ynwa-integration-testing/src/compare.rs:448): `→ true` не ломает тесты.
Для `RestartSet` проверяется только совпадающая позиция
([`compare_tests.rs`](ynwa-integration-testing/src/tests/compare_tests.rs:497)); несовпадение
позиции `RestartSet` не тестируется. Нужен тест с другой `position`.

### 5. Режим `subsequence` не проверяет порядок при нескольких ожиданиях и продвижение курсора

В [`compare_journal`](ynwa-integration-testing/src/compare.rs:93) замена
`cursor = index + 1` на `cursor = index` не ломает ни один тест: оба существующих
subsequence-теста используют одно ожидаемое событие
([`subsequence_skips_unlisted_events`](ynwa-integration-testing/src/tests/compare_tests.rs:132),
[`subsequence_reports_missing_event`](ynwa-integration-testing/src/tests/compare_tests.rs:155)).
Не проверен случай двух и более ожидаемых записей в режиме `subsequence`, в том числе
повторяющихся одинаковых. Нужен тест с несколькими `[[expect.journal]]` и `journal_match = "subsequence"`.

## Итог

Существенная часть нового кода покрыта тестами надёжно (мутации ловятся). Найденные проблемы —
локальные пробелы в отрицательных ветвях сверки ожиданий (`compare.rs`) и в режиме
`subsequence`. Считаю замечания **существенными**: без них неверно заданное ожидание сценария в
указанных полях не приводит к падению теста, что подрывает защиту от регресса — основную цель
задачи. Предлагаю вернуться к кодированию (шаг 4) и добавить перечисленные отрицательные тесты.

# Ревью архитектурного решения — task_8 (шаг 3)

Статус: **замечания значительные** → после апрува человека вернуться к шагу 2.

## Что проверено и подтверждено

- Кодировки значений корректны: [`Point3D`](ynwa-core/src/field/zones.rs:11) и
  [`Velocity3D`](ynwa-core/src/field/zones.rs:22) сериализуются как `{x,y,z}` числами в базовых
  единицах (тест [`point3d_is_serialized_in_meters`](ynwa-core/src/tests/serde_tests.rs:114));
  [`Team`](ynwa-core/src/team.rs:3) — как `"A"`/`"B"`.
- Словарь журнала §3.3 соответствует [`JournalEvent`](ynwa-core/src/journal.rs:25);
  `External { kind: "football_event" }` подтверждён константой
  [`FOOTBALL_EVENT_KIND`](ynwa-football/src/game_manager.rs:42).
- [`CollectJournalRecorder`](ynwa-core/src/journal.rs:120) /
  [`EventsCollection`](ynwa-core/src/journal.rs:87) и
  [`decode_football_events`](ynwa-football/src/replay.rs:56) уже существуют и вписываются в план.
- [`create_football_world`](ynwa-football/src/lib.rs:240) использует температуру `0.7`
  ([`PLAYER_RNG_TEMPERATURE`](ynwa-football/src/lib.rs:38)); обёртка над билдером реализуема.
- Поля [`GameState`](ynwa-core/src/game.rs:261) публичны — применение частичного снапшота реализуемо.
- [`FootballEvent`](ynwa-football/src/events.rs:9) совпадает с типами стоп-критериев и ожиданий.
- Циклов зависимостей нет; разделение с task_4 в основном согласовано с
  [`task_4`](context/tasks/task_4_serialization_snapshots/task.md:32).

Подход в целом соответствует заданию, разделение ответственности (core / football /
интеграционный крейт) корректное.

## Значительные замечания

### A1. «Тихий» откат Lua-инициализации в placeholder ломает смысл теста

Общая сборка систем [`add_football_systems`](ynwa-football/src/lib.rs:209) при ошибке
[`ScriptedDecisionMaker::new`](ynwa-football/src/lib.rs:214) печатает предупреждение и молча
подставляет placeholder-генератор решений. Если раннер собирает мир через тот же билдер, сценарий
с битым Lua-скриптом **пройдёт** (игроки получат случайные placeholder-решения) вместо падения —
это противоположно цели «защита от регресса».

Требуется: строгий режим сборки для тестов — любая ошибка скрипта/инициализации движка решений
должна приводить к ошибке загрузки сценария, а не к откату на placeholder.

### A2. Порядок применения снапшота противоречит заявленной расстановке по умолчанию

Билдер собирает мир в стадии `Setup("kick off")` ([`create_football_world`](ynwa-football/src/lib.rs:282)),
поэтому [`Game::with_stage`](ynwa-core/src/game.rs:289) расставляет всех игроков за полем.
Снапшот применяется **после** сборки и переопределяет позиции только перечисленных игроков, а
неперечисленные останутся за полем, а не «в центре региона `start` в Play», как обещают §3.1/§4.

Требуется одно из: (а) билдер принимает целевую стадию до расстановки, (б) применение стадии
перерасставляет не указанных в снапшоте игроков, (в) честно задокументировать фактическое поведение.

### A3. Патч-тип не содержит `score`, хотя `final_state` его сверяет

§6 описывает патч-тип полями stage/мяч/игроки/setup, а §5.2 утверждает, что финальное состояние
«реализуется тем же типом частичного состояния». При этом [`final_state.toml`](context/tasks/task_8_integration_testing/task.md:233)
дополнительно несёт счёт.

Требуется: либо патч-тип включает `score` (с семантикой «в снапшоте игнорируется, в сверке
проверяется»), либо сверка счёта выполняется отдельным механизмом — и тогда формулировка
«тем же типом» должна быть уточнена.

### S1. Конфликт полей снапшота со сбросом в стадии `Setup`

В первый шаг [`FootballGameManager::update`](ynwa-football/src/game_manager.rs:56) в стадии `Setup`
безусловно перезаписывает мяч: `position = restart_position или initial`, `velocity = 0`,
`possessed_by = None`, `last_possessing_team = None`. Требование же допускает
`stage = "Setup"` + `possessed_by` / `last_possessing_team` / `ball.position` — эти поля будут
молча затёрты на первом шаге.

Требуется: (а) запрещать задание владения/позиции мяча при `stage = "Setup"` с ошибкой загрузки,
(б) документировать игнорирование, либо (в) применять снапшот после первого Setup-тика.

### S2. Коллизия имён `Snapshot` между task_8 и task_4

task_8 добавляет в `ynwa-core` частичный тип `Snapshot` (Option-поля), а task_4 —
[`record::Snapshot { tick, timestamp, journal_cursor, state }`](context/tasks/task_4_serialization_snapshots/task.md:32).
Два разных типа с одним именем в одном крейте — конфликт и путаница.

Требуется: переименовать частичный тип (например, `StatePatch` / `PartialState`).

### S3. Не задано единое правило адресации игроков

TOML снапшота адресует игроков `{team, number}`, ядро — глобальным индексом
([`JournalEvent::DecisionAssigned`](ynwa-core/src/journal.rs:26) использует `player_index: usize`,
`PossessionChange` — `possessed_by: Option<usize>`). В §3.3 для `decision_assigned` / `kick_outcome` /
`possession_change` поле названо `player` / `possessed_by` без указания, чем адресовать: глобальным
индексом (хрупко, зависит от порядка [`FsTeamRepository`](ynwa-repository/src/fs_team_repository.rs:93))
или `{team, number}`.

Требуется: единое и явное правило адресации для всех ожиданий журнала.

## Незначительные замечания

### S4. Несогласованность `possessed_by` между файлами

В [`initial_state.toml`](context/tasks/task_8_integration_testing/task.md:121) — `{team, number}` и
`"none"`; в [`final_state.toml`](context/tasks/task_8_integration_testing/task.md:237) показан только
`"none"`. Задать одинаково все формы (конкретный игрок / `"none"` / отсутствие) для обоих файлов.

### S5. Не определена проверка причины в `Setup` для стоп-критерия и ожиданий

`when = "stage"`, `stage = "Setup"` — не сказано, матчится ли `Setup` любой причины или конкретной,
и как причину задавать (аналогично для `[expect.stop]` и `final_state.stage`).

### S6. Не указаны регистрация крейта и путь к преамбулам

Не упомянуто добавление `ynwa-integration-testing` в [`members`](Cargo.toml:3) workspace и способ
нахождения `ynwa-scripts/preambles` (существующие тесты используют хрупкий `../ynwa-scripts/preambles`).

### S7. Хрупкость точного списка журнала

`[[expect.journal]]` требует точного совпадения длины и порядка. Соответствует исходному требованию
«точный упорядоченный список», но стоит оговорить, что сценарии со стартом в `Setup` включают и
`DecisionAssigned` от менеджера расстановки, и предусмотреть в будущем режим «подпоследовательность».

### A4. Не описан вход в тесты (harness)

Описаны раннер и сверка, но не сам `#[test]`: обход `scenarios/`, фильтр одного сценария для
отладки, отображение структурной ошибки в выводе `cargo test`.

### A5. Побочный вывод в stdout при сборке

[`create_football_world`](ynwa-football/src/lib.rs:216) и ветка ошибки печатают в stdout/stderr;
тестовый раннер не должен шуметь — логирование убрать из пути сборки тестового мира.

## Рекомендация

Вернуться к шагу 2, устранить значительные замечания A1–A3, S1–S3 (и по согласованию —
незначительные S4–S7, A4–A5), получить апрув человека, после чего стереть этот файл (он остаётся
в истории).

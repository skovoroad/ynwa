# Добавление решения "Удар по мячу" (Kick Decision)

## Дата: 5 февраля 2026

## Описание изменений

Добавлен новый тип решения игрока: **Kick** (удар по мячу).

### Структура решения

```rust
pub enum Decision {
    Run(DecisionTarget),
    Stop,
    Kick(Point3D), // NEW: Kick the ball towards target point
}
```

Решение `Kick` хранит точку (`Point3D`), куда направлен удар.

## Изменённые файлы

### 1. **ynwa-core/src/game.rs**
- Добавлен вариант `Decision::Kick(Point3D)`
- Обновлена функция `convert_decision_to_display_orientation()` для обработки Kick
- Добавлена поддержка переворота координат для команды B

### 2. **ynwa-core/src/systems/action.rs**
- Добавлена обработка `Decision::Kick` в `ActionSystem`
- На данный момент Kick просто останавливает игрока (placeholder для будущей реализации)
- TODO: Реализовать механику удара по мячу

### 3. **ynwa-player/src/ui.rs**
- Добавлено отображение решения "Kick" в UI
- Показывается как "Kick" без деталей точки

### 4. **ynwa-decisions/src/lua_format.rs**
- Добавлен вариант `LuaDecision::Kick { target }`
- Обновлена документация Lua контракта
- Добавлен тест `test_lua_decision_kick()`

### 5. **ynwa-core/src/systems/decision/scripted_decision_maker.rs**
- Добавлена обработка action = "kick" в парсере
- Добавлена функция `parse_point_target_direct()` для парсинга точки без DecisionTarget
- Добавлен тест `test_json_decision_maker_kick()`

### 6. **ynwa-core/src/systems/decision/json_decision_maker.rs**
- Добавлена обработка action = "kick" в парсере
- Добавлена функция `parse_point_target_direct()`
- Добавлен тест `test_json_decision_maker_kick()`

### 7. **ynwa-script-tests/tests/basic_scripts.rs**
- Обновлены pattern matching для поддержки нового варианта Decision::Kick

## Lua API

Для создания решения "удар" из Lua скрипта:

```lua
function make_decision()
    return {
        action = "kick",
        target = {x = 50.0, z = 30.0}
    }
end
```

### Параметры:
- `action`: строка `"kick"`
- `target`: таблица с полями:
  - `x`: координата X точки (обязательно)
  - `z`: координата Z точки (обязательно)
  - `y`: координата Y точки (опционально, по умолчанию 0.0)

## Координаты

Координаты удара передаются в **ориентации игрока** (как и для Run).
- Для команды A координаты остаются без изменений
- Для команды B координаты автоматически переворачиваются в `DecisionSystem`

## Текущее поведение

**Важно:** В данный момент решение Kick **не реализовано функционально**.
- `ActionSystem` обрабатывает Kick, но просто останавливает игрока
- Мяч не получает импульс и не летит в заданном направлении
- Требуется будущая реализация физики удара

## Тесты

Добавлено **3 новых теста**:
1. `ynwa-decisions::lua_format::test_lua_decision_kick` - тест сериализации Lua формата
2. `ynwa-core::decision::scripted_decision_maker::test_json_decision_maker_kick` - тест ScriptedDecisionMaker
3. `ynwa-core::decision::json_decision_maker::test_json_decision_maker_kick` - тест JsonDecisionMaker

Все существующие тесты обновлены для поддержки нового варианта.

## Статистика

- **Всего тестов:** 211 (156 в ynwa-core + 47 в ynwa-decisions + 2 в ynwa-script-tests + 6 doc tests)
- **Новых тестов:** 3
- **Обновлённых тестов:** ~5
- **Изменённых файлов:** 7
- **Все тесты:** ✅ PASSED

## Следующие шаги

1. Реализовать физику удара в `ActionSystem`:
   - Проверка владения мячом
   - Расчёт траектории и силы удара на основе `shot_power` и `shot_accuracy`
   - Применение импульса к мячу
   
2. Добавить визуализацию удара в UI:
   - Показать траекторию удара
   - Отобразить точку назначения

3. Расширить контекст Lua:
   - Добавить информацию о владении мячом
   - Добавить расстояние до мяча
   - Добавить информацию о положении ворот

4. Создать тестовые скрипты с использованием Kick для проверки поведения

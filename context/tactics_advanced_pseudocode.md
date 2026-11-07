# Псевдокод тактики «остроатакующая команда» (черновик для анализа)

Назначение: предварительный код/псевдокод, который использует **предлагаемые** функции
стандартной библиотеки и реализует тактику из `context/tactrics_new.md`. Это **не** рабочий
код — часть функций ещё не реализована в `core.lua`/`stdlib.lua`. Анализируя этот файл, мы
уточняем список доработок стандартной библиотеки.

Обозначения:

- `✅` — функция уже есть в `core.lua`/`stdlib.lua`.
- `➕` — предлагаемая функция (пока отсутствует).
- `⚠️` — поведение/точка требует решения.

Ограничения будущего DSL соблюдаются: только `if/elseif/else`, локальные переменные,
без циклов, без координат, без собственных типов; вся итерация и работа с координатами
спрятаны в stdlib.

---

## 1. Новые предикаты владения мячом

`make_decision()` **не меняется**. Вводятся два предиката:

| Функция | Семантика | Значение |
|---|---|---|
| ➕ `is_ball_free()` | мяч в свободном полёте (никто физически не держит) | `ball_owner() == nil` |
| ➕ `my_team_has_ball()` | команда владеет мячом (по последнему владельцу) | `get_ball_owner_team() == my_team_name()` |

`is_ball_free()` опирается на текущего держателя (`ball_owner()` / `possessed_by`),
а `my_team_has_ball()` — на команду, владевшую мячом последней
(`get_ball_owner_team()` / `last_possessing_team`). Предикаты **не взаимоисключающие**:
во время паса своей команды оба истинны.

---

## 2. Предлагаемые функции stdlib (сводка)

**Предикаты (проверка состояния → boolean):**

| Функция | Примечание |
|---|---|
| ➕ `is_ball_free()` | «мяч в свободном полёте» |
| ➕ `my_team_has_ball()` | «команда владеет» |
| ➕ `am_i_topN_closest_to_ball(n)` | обобщение `am_i_top3_closest_to_ball` |
| ➕ `is_free(position)` | нет оппонента в радиусе |
| ➕ `is_ahead_of_me(position)` | «впереди» (больше по Z) |
| ➕ `has_free_teammate_ahead()` | есть свободный впереди (среди всех товарищей) |
| ➕ `has_free_teammate_ahead_of(numbers)` | есть свободный впереди среди номеров |

**Селекторы (→ одна сущность):**

| Функция | Примечание |
|---|---|
| ➕ `nearest_free_teammate_ahead()` | ближайший свободный впереди |
| ➕ `nearest_free_teammate_ahead_of(numbers)` | ближайший свободный впереди среди номеров |

**Решения (→ decision):**

| Функция | Примечание |
|---|---|
| ➕ `pass_to_nearest_free_teammate_ahead()` | пас ближайшему свободному впереди |
| ➕ `pass_to_nearest_free_teammate_ahead_of(numbers)` | пас ближайшему свободному впереди среди номеров |

**Доступ к состоянию (`core.lua`):**

| Функция | Примечание |
|---|---|
| ➕ `get_opponents()` | оппоненты `{index, number, position}` (данные уже в контексте) |

---

## 3. preamble.lua (командная преамбула)

```lua
-- Остроатакующая команда: preamble.lua
-- Списки номеров — данные команды, задаются пользователем (не сущность ядра).
FORWARDS = {9, 10, 11}   -- нападающие (пример)
WINGERS  = {7, 11}       -- крайние полузащитники (пример)

-- Общая логика: рядом с мячом — атаковать соперника, иначе в позицию.
function press_or_attack()
    if am_i_topN_closest_to_ball(3) then   -- ➕
        return chase_ball()                -- ✅ «атаковать соперника»
    end
    return run_to_attack_position()        -- ✅
end

function press_or_defend()
    if am_i_topN_closest_to_ball(3) then   -- ➕
        return chase_ball()                -- ✅
    end
    return run_to_defence_position()       -- ✅
end

team_play = {
    ball_is_free      = press_or_defend,
    team_has_ball     = press_or_attack,
    opponent_has_ball = press_or_defend,
}

-- Вратарь: различает «свободный полёт» и «кто-то владеет» через is_ball_free().
function goalkeeper_position()
    if is_ball_free() then                     -- ➕ «мяч в свободном полёте»
        return goalkeeper_free_ball_action()   -- ⚠️ поведение уточняется
    end
    -- кто-то владеет мячом — на линии мяча, не выходя за линию ворот
    return default_goalkeeper_cover_position() -- ✅
end

goalkeeper_play = {
    i_have_ball       = goalkeeper_distribute, -- ⚠️ не задано в tactrics_new.md
    team_has_ball     = goalkeeper_position,
    opponent_has_ball = goalkeeper_position,
    ball_is_free      = goalkeeper_position,
}
```

---

## 4. script.lua по ролям (переопределяют `i_have_ball`)

### Крайний защитник

```lua
player_play = {
    i_have_ball = function()
        if is_in_opponent_penalty_area() then                       -- ✅
            return pass_to_nearest_free_teammate_ahead_of(FORWARDS) -- ➕
        end
        return run_to_opponent_penalty_area()                       -- ✅
    end,
}
```

### Центральный защитник

```lua
player_play = {
    i_have_ball = function()
        if has_free_teammate_ahead() then                -- ➕
            return pass_to_nearest_free_teammate_ahead() -- ➕
        end
        return run_to_attack_position()                  -- ✅
    end,
}
```

### Крайний полузащитник

```lua
player_play = {
    i_have_ball = function()
        if has_free_teammate_ahead_of(FORWARDS) then               -- ➕
            return pass_to_nearest_free_teammate_ahead_of(FORWARDS) -- ➕
        end
        if is_in_opponent_penalty_area() then                      -- ✅
            return kick_to_opponent_goal()                         -- ✅
        end
        return run_to_opponent_penalty_area()                      -- ✅
    end,
}
```

### Центральный полузащитник

```lua
player_play = {
    i_have_ball = function()
        if has_free_teammate_ahead_of(FORWARDS) then                -- ➕
            return pass_to_nearest_free_teammate_ahead_of(FORWARDS) -- ➕
        end
        if has_free_teammate_ahead_of(WINGERS) then                 -- ➕
            return pass_to_nearest_free_teammate_ahead_of(WINGERS)  -- ➕
        end
        if is_in_opponent_penalty_area() then                       -- ✅
            return kick_to_opponent_goal()                          -- ✅
        end
        return run_to_opponent_penalty_area()                       -- ✅
    end,
}
```

### Нападающий

```lua
player_play = {
    i_have_ball = function()
        if is_in_opponent_penalty_area() then -- ✅
            return kick_to_opponent_goal()    -- ✅
        end
        return run_to_opponent_penalty_area() -- ✅
    end,
}
```

---

## 5. Открытые вопросы (для выработки предложений по stdlib)

1. `goalkeeper_free_ball_action()` — что делает вратарь при свободном мяче: остаётся на
   линии или идёт подбирать (`chase_ball()`)? В `tactrics_new.md` это не определено.
2. Списки номеров (`FORWARDS`, `WINGERS`) — где хранить: в `preamble.lua` как константы
   или в `tactical.toml` (конфиг)? Влияет на DSL (список номеров — единственный «контейнер»,
   который, возможно, придётся поддержать как литерал).
3. Нужны ли отдельные функции «без списка» (`nearest_free_teammate_ahead`) и «со списком»
   (`nearest_free_teammate_ahead_of`) — или свести к одной со списком?
4. Точные радиусы/пороги для `is_free` и `am_i_topN_closest_to_ball` (значения по умолчанию).
5. ~~Семантика `my_team_has_ball()`~~ — решено: команда владеет по последнему владельцу
   (`last_possessing_team`), а не по текущему держателю; пас не меняет владение.

# Proposal

## Why

Only two `stdlib.lua` actions (`stop`, `kick_to_position`) accept a `reason` argument; the rest hard-code their reason string. Teams cannot tag their decisions with a custom reason, so they cannot grep or analyze the recorded journal by their own decision intent.

## What Changes

- Add an optional trailing `reason` argument to every `stdlib.lua` action that currently hard-codes a reason, defaulting to the existing built-in string when omitted.
- When a custom reason is provided and the action also carries a built-in (or composed) reason, join both with a stable `" | "` separator so each remains independently greppable.
- Leave `stop` and `kick_to_position` unchanged: they already accept a plain `reason` and use it verbatim.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `scripting-and-protocols`: `stdlib.lua` action functions gain an optional `reason` argument and a concatenation rule for composed reasons.

## Impact

- `ynwa-scripts/preambles/stdlib.lua` — action signatures and a `compose_reason` helper.
- `ynwa-script-tests/tests/stdlib_functions.rs` — coverage for custom and composed reasons.
- `scripting-and-protocols` spec — stdlib action signature contract.

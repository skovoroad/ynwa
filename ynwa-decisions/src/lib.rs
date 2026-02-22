//! Decision-making system for game AI.
//!
//! This library provides a game-agnostic decision-making system based on Lua scripting.
//! It communicates with the game engine through JSON, avoiding tight coupling.
//!
//! # Architecture
//!
//! - **Input:** JSON context with game state
//! - **Output:** JSON decision with player action
//! - **Initialization:** JSON config with player scripts
//!
//! # JSON Contract
//!
//! The library uses `serde_json::Value` as lingua franca, allowing independent
//! versioning of the game engine and decision system.
//!
//! # Lua Sandbox
//!
//! Dangerous globals disabled: `io`, `os`, `package`, `require`, `load`, `loadfile`,
//! `dofile`, `debug`, `collectgarbage`, `_G`.
//!
//! # Timeout
//!
//! Enabled via `DecisionEngine::new(Some(Duration))`. Hook fires every 10,000 instructions (~2x overhead).
//! Script code reloads on each `execute()` call — no persistent state between decisions.

mod decision_engine;
mod lua_executor;
mod lua_format;

pub use decision_engine::{DecisionEngine, DecisionEngineError};
pub use lua_executor::{LuaExecutor, ScriptError, ScriptResult};

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

mod decision_engine;
mod lua_executor;
mod lua_format;

pub use decision_engine::{DecisionEngine, DecisionEngineError};
pub use lua_executor::{LuaExecutor, ScriptError, ScriptResult};

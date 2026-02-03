//! Scripting system for game-agnostic decision making.
//!
//! This module provides a Lua-based scripting system that can be used
//! across different games. Game-specific functionality is provided through
//! preamble code (as a plain string) that extends the base scripting environment.

mod lua_executor;

pub use lua_executor::{LuaExecutor, ScriptError, ScriptResult};

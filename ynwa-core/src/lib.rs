/// YNWA Football Manager - Core Library
///
/// This is the game engine core that handles all game logic.

/// Returns the core version
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Core module initialization
pub fn init() {
    println!("YNWA Core initialized (version {})", version());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}

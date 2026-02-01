/// System trait - interface for all game systems.
pub trait System {
    fn update(&mut self, game: &mut crate::game::Game, timestamp: f32);
}

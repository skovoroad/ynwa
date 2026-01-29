use ynwa_core::{football::create_football_field, Game, GameConfig};
use uom::si::length::meter;

fn main() {
    println!("YNWA - Football Manager\n");
    
    // Create field
    let field = create_football_field();
    
    println!("Field dimensions:");
    println!("  Length: {} m", field.length().get::<meter>());
    println!("  Width: {} m\n", field.width().get::<meter>());
    
    // Create game
    let mut game = Game::new(GameConfig { field });
    println!("Game initialized. Time: {:.1}s\n", game.state().elapsed_time);
    
    // Simulate a few steps
    println!("Running simulation...");
    for i in 1..=5 {
        let events = game.step(1.0); // 1 second per step
        println!("  Step {}: {:.1}s (events: {})", i, game.state().elapsed_time, events.len());
    }
    
    println!("\nSimulation complete!");
}

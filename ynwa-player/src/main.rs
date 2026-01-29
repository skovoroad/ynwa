/// YNWA Football Manager - Player Application
///
/// Local client that uses the core library for game simulation

fn main() {
    println!("YNWA Player Application");
    
    // Initialize core
    ynwa_core::init();
    
    println!("Using core version: {}", ynwa_core::version());
    println!("Ready to play!");
}

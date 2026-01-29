use ynwa_core::football::create_football_field;

fn main() {
    println!("YNWA Core Test - Football Field Zones");
    
    let field = create_football_field();
    
    println!("\nField dimensions:");
    println!("  Length: {} m", field.length().get::<uom::si::length::meter>());
    println!("  Width: {} m", field.width().get::<uom::si::length::meter>());
    
    println!("\nZones ({} total):", field.zones().len());
    let mut zone_names: Vec<_> = field.zones().keys().collect();
    zone_names.sort();
    for (name, team) in zone_names {
        println!("  - {:?}: {}", team, name);
    }
}

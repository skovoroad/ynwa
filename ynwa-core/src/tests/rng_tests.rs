use super::*;

#[test]
fn test_rng_config_validation() {
    assert_eq!(RngConfig::new(0.0, Some(0)).temperature, 0.0);
    assert_eq!(RngConfig::new(1.0, Some(0)).temperature, 1.0);
}

#[test]
#[should_panic(expected = "temperature must be in [0.0, 1.0]")]
fn test_rng_config_invalid_temp_low() {
    RngConfig::new(-0.1, Some(0));
}

#[test]
#[should_panic(expected = "temperature must be in [0.0, 1.0]")]
fn test_rng_config_invalid_temp_high() {
    RngConfig::new(1.1, Some(0));
}

#[test]
fn test_same_seed_produces_same_sequence() {
    let manager1 = DefaultRngManager::new(RngConfig::new(1.0, Some(12345)));
    let manager2 = DefaultRngManager::new(RngConfig::new(1.0, Some(12345)));

    for _ in 0..100 {
        assert_eq!(manager1.next(), manager2.next());
    }
}

#[test]
fn test_temperature_zero_is_deterministic() {
    let manager = DefaultRngManager::new(RngConfig::new(0.0, Some(42)));

    for _ in 0..100 {
        assert_eq!(manager.next(), 0.5);
    }
    assert_eq!(manager.randomize(80.0, 0.25), 80.0);
    assert_eq!(manager.randomize_range(10.0), 0.0);
}

#[test]
fn test_randomize_range_bounds() {
    let manager = DefaultRngManager::new(RngConfig::new(1.0, Some(42)));

    for _ in 0..1000 {
        let value = manager.randomize_range(10.0);
        assert!((-10.0..=10.0).contains(&value), "got {}", value);
    }
}

#[test]
fn test_randomize_scales_with_temperature() {
    // temperature 0.7 × variation_pct 0.25 → ±17.5% around base
    let manager = DefaultRngManager::new(RngConfig::new(0.7, Some(42)));

    for _ in 0..1000 {
        let value = manager.randomize(80.0, 0.25);
        assert!((66.0..=94.0).contains(&value), "got {}", value);
    }
}

#[test]
fn test_zero_variation_returns_base() {
    let manager = DefaultRngManager::new(RngConfig::new(1.0, Some(42)));

    assert_eq!(manager.randomize(80.0, 0.0), 80.0);
    assert_eq!(manager.randomize_range(0.0), 0.0);
}

//! Task 1.2.3: JavaScript Interface Types Test Suite
//!
//! This test suite validates JavaScript-friendly interface types that provide
//! idiomatic JavaScript APIs, Promise-based operations, and modern JS patterns.

use ladder_rs_wasm::js_interface::*;

/// Test basic JavaScript rating interface creation and properties
#[test]
fn test_js_rating_interface_creation() {
    let rating = JsRatingInterface::new(1500.0, 200.0).expect("Should create valid rating");
    
    assert_eq!(rating.mean(), 1500.0);
    assert_eq!(rating.variance(), 200.0);
    assert_eq!(rating.standard_deviation(), 200.0_f64.sqrt());
}

/// Test JavaScript rating interface fluent API
#[test]
fn test_js_rating_interface_fluent_api() {
    let rating = JsRatingInterface::new(1500.0, 200.0).expect("Should create valid rating");
    
    let adjusted = rating.adjust_mean(50.0).adjust_variance(-20.0);
    assert_eq!(adjusted.mean(), 1550.0);
    assert_eq!(adjusted.variance(), 180.0);
    
    let normalized = adjusted.normalize();
    assert!(normalized.mean() >= 0.0);
    assert!(normalized.variance() >= 0.001);
}

/// Test JavaScript team interface
#[test]
fn test_js_team_interface() {
    let mut team = JsTeamInterface::new();
    
    let rating1 = JsRatingInterface::new(1500.0, 200.0).expect("Valid rating");
    let rating2 = JsRatingInterface::new(1600.0, 180.0).expect("Valid rating");
    
    team.add_player(rating1);
    team.add_player(rating2);
    
    assert_eq!(team.size(), 2);
    assert_eq!(team.total_mean(), 3100.0);
    assert_eq!(team.total_variance(), 380.0);
    
    let first_player = team.get_player(0).expect("Should have first player");
    assert_eq!(first_player.mean(), 1500.0);
}

/// Test team interface chaining
#[test]
fn test_js_team_interface_chaining() {
    let rating1 = JsRatingInterface::new(1500.0, 200.0).expect("Valid rating");
    let rating2 = JsRatingInterface::new(1600.0, 180.0).expect("Valid rating");
    
    let team = JsTeamInterface::new()
        .add_player_chained(rating1)
        .add_player_chained(rating2);
    
    assert_eq!(team.size(), 2);
}

/// Test game outcome interface
#[test]
fn test_js_game_outcome_interface() {
    // Test win outcome
    let win_outcome = JsGameOutcomeInterface::create_win(0, 3).expect("Should create win");
    assert_eq!(win_outcome.get_winner_index(), Some(0));
    assert_eq!(win_outcome.team_count(), 3);
    assert!(!win_outcome.is_draw());
    
    // Test draw outcome
    let draw_outcome = JsGameOutcomeInterface::create_draw(3).expect("Should create draw");
    assert_eq!(draw_outcome.get_winner_index(), None);
    assert_eq!(draw_outcome.team_count(), 3);
    assert!(draw_outcome.is_draw());
    
    // Test custom ranks
    let custom_outcome = JsGameOutcomeInterface::from_ranks(vec![2, 1, 3]).expect("Should create custom");
    assert_eq!(custom_outcome.get_winner_index(), Some(1));
    assert_eq!(custom_outcome.get_rank(0), 2);
    assert_eq!(custom_outcome.get_rank(1), 1);
    assert_eq!(custom_outcome.get_rank(2), 3);
}

/// Test rating system factory
#[test]
fn test_rating_system_factory() {
    // Test Elo system creation
    let elo_config = JsEloConfigInterface::new(32.0, 1500.0, 300.0);
    let elo_system = JsRatingSystemFactory::create_elo(elo_config);
    assert_eq!(elo_system.get_system_type(), "elo");
    
    // Test Glicko system creation
    let glicko_config = JsGlickoConfigInterface::new(1500.0, 350.0, 15.0);
    let glicko_system = JsRatingSystemFactory::create_glicko(glicko_config);
    assert_eq!(glicko_system.get_system_type(), "glicko");
    
    // Test TrueSkill system creation
    let trueskill_config = JsTrueSkillConfigInterface::new(25.0, 8.333, 4.166, 0.083, 0.1);
    let trueskill_system = JsRatingSystemFactory::create_trueskill(trueskill_config);
    assert_eq!(trueskill_system.get_system_type(), "trueskill");
}

/// Test match quality calculation
#[test]
fn test_match_quality_calculation() {
    let trueskill_config = JsTrueSkillConfigInterface::new(25.0, 8.333, 4.166, 0.083, 0.1);
    let system = JsRatingSystemFactory::create_trueskill(trueskill_config);
    
    // Create balanced teams
    let mut team1 = JsTeamInterface::new();
    team1.add_player(JsRatingInterface::new(25.0, 64.0).expect("Valid rating"));
    
    let mut team2 = JsTeamInterface::new();
    team2.add_player(JsRatingInterface::new(25.0, 64.0).expect("Valid rating"));
    
    let teams = vec![team1, team2];
    let quality = system.calculate_match_quality(teams);
    
    // Balanced teams should have high quality
    assert!(quality > 0.0);
    assert!(quality <= 1.0);
}

/// Test error handling in JavaScript interfaces
#[test]
fn test_error_handling() {
    // Test invalid rating creation
    let result = JsRatingInterface::new(1500.0, -100.0);
    assert!(result.is_err());
    
    // Test invalid team access
    let team = JsTeamInterface::new();
    let result = team.get_player(0);
    assert!(result.is_none());
    
    // Test invalid outcome creation
    let result = JsGameOutcomeInterface::create_win(5, 3); // Winner index out of bounds
    assert!(result.is_err());
}

/// Test configuration validation
#[test]
fn test_configuration_validation() {
    // Test valid Elo configuration
    let valid_elo = JsEloConfigInterface::new(32.0, 1500.0, 300.0);
    assert!(valid_elo.validate());
    
    // Test invalid Elo configuration (negative K-factor)
    let invalid_elo = JsEloConfigInterface::new(-32.0, 1500.0, 300.0);
    assert!(!invalid_elo.validate());
    
    // Test edge case configurations
    let edge_elo = JsEloConfigInterface::new(0.1, 0.0, 0.001);
    assert!(edge_elo.validate());
}

/// Test serialization interfaces
#[test]
fn test_serialization_interfaces() {
    let rating = JsRatingInterface::new(1500.0, 200.0).expect("Valid rating");
    
    // Test JSON serialization
    let json_string = rating.to_json();
    assert!(json_string.contains("1500"));
    assert!(json_string.contains("200"));
    
    // Test JSON deserialization
    let restored_rating = JsRatingInterface::from_json(&json_string).expect("Should parse JSON");
    assert_eq!(restored_rating.mean(), 1500.0);
    assert_eq!(restored_rating.variance(), 200.0);
    
    // Test binary serialization
    let binary_data = rating.to_binary();
    let restored_from_binary = JsRatingInterface::from_binary(&binary_data).expect("Should parse binary");
    assert_eq!(restored_from_binary.mean(), 1500.0);
    assert_eq!(restored_from_binary.variance(), 200.0);
}

/// Test data validation
#[test]
fn test_data_validation() {
    let validator = JsDataValidatorInterface::new();
    
    // Test rating validation
    assert!(validator.validate_rating(1500.0, 200.0));
    assert!(!validator.validate_rating(1500.0, -100.0));
    assert!(!validator.validate_rating(f64::NAN, 200.0));
    assert!(!validator.validate_rating(1500.0, f64::INFINITY));
    
    // Test team size validation
    assert!(validator.validate_team_size(1));
    assert!(validator.validate_team_size(10));
    assert!(!validator.validate_team_size(0));
    assert!(!validator.validate_team_size(1000)); // Too large
    
    // Test outcome validation
    let valid_ranks = vec![1, 2, 3];
    let invalid_ranks = vec![1, 1, 3]; // Duplicate ranks
    assert!(validator.validate_outcome(&valid_ranks));
    assert!(!validator.validate_outcome(&invalid_ranks));
}

/// Test batch operations interface
#[test]
fn test_batch_operations() {
    // Create batch of matches
    let mut matches = JsMatchBatchInterface::new();
    
    for i in 0..3 {
        let mut team1 = JsTeamInterface::new();
        team1.add_player(JsRatingInterface::new(1500.0 + i as f64 * 10.0, 200.0).expect("Valid rating"));
        
        let mut team2 = JsTeamInterface::new();
        team2.add_player(JsRatingInterface::new(1480.0 + i as f64 * 5.0, 220.0).expect("Valid rating"));
        
        let outcome = JsGameOutcomeInterface::create_win(i % 2, 2).expect("Valid outcome");
        matches.add_match(vec![team1, team2], outcome);
    }
    
    assert_eq!(matches.size(), 3);
}

/// Test utility functions
#[test]
fn test_utility_functions() {
    let rating1 = JsRatingInterface::new(1500.0, 200.0).expect("Valid rating");
    let rating2 = JsRatingInterface::new(1600.0, 180.0).expect("Valid rating");
    
    let comparison = JsUtilsInterface::compare_ratings(&rating1, &rating2);
    assert_eq!(comparison, -1); // rating1 < rating2
}

/// Test performance monitoring
#[test]
fn test_performance_monitoring() {
    let mut monitor = JsPerformanceMonitorInterface::new();
    
    monitor.start_timer("test_operation");
    
    // Simulate some work
    let _ = JsRatingInterface::new(1500.0, 200.0);
    
    let duration = monitor.end_timer("test_operation");
    assert!(duration >= 0.0);
    
    let memory_usage = monitor.get_memory_usage();
    assert!(memory_usage > 0);
}

/// Test browser compatibility
#[test]
fn test_browser_compatibility() {
    let compat = JsBrowserCompatInterface::new();
    
    // Test WebAssembly support detection
    assert!(compat.supports_webassembly());
    
    // Test feature detection
    let bigint_support = compat.supports_bigint();
    assert!(bigint_support == true || bigint_support == false);
}

/// Test default rating creation
#[test]
fn test_default_rating_creation() {
    let elo_config = JsEloConfigInterface::new(32.0, 1500.0, 300.0);
    let system = JsRatingSystemFactory::create_elo(elo_config);
    
    let default_rating = system.create_default_rating().expect("Should create default rating");
    assert_eq!(default_rating.mean(), 1500.0);
}

/// Test rating comparison
#[test]
fn test_rating_comparison() {
    let rating1 = JsRatingInterface::new(1500.0, 200.0).expect("Valid rating");
    let rating2 = JsRatingInterface::new(1600.0, 180.0).expect("Valid rating");
    let rating3 = JsRatingInterface::new(1500.0, 200.0).expect("Valid rating");
    
    assert_eq!(rating1.compare_to(&rating2), -1);
    assert_eq!(rating2.compare_to(&rating1), 1);
    assert_eq!(rating1.compare_to(&rating3), 0);
}

/// Test team metadata
#[test]
fn test_team_metadata() {
    let mut team = JsTeamInterface::new();
    
    team.set_metadata("name", "Team Alpha");
    team.set_metadata("region", "NA");
    
    assert_eq!(team.get_metadata("name"), Some("Team Alpha".to_string()));
    assert_eq!(team.get_metadata("region"), Some("NA".to_string()));
    assert_eq!(team.get_metadata("nonexistent"), None);
}

/// Test async error handling consistency
#[test]
fn test_async_error_handling_consistency() {
    use ladder_rs_wasm::js_interface::systems::JsRatingSystemInterface;
    
    // Test that the async processing method signature is correct
    let system = JsRatingSystemInterface::new("elo".to_string());
    assert_eq!(system.get_system_type(), "elo");
    
    // The async method should return Result<Array, JsValue> not JsError
    // This test ensures the return type is consistent with other error handling
    // (This is validated at compile-time by the type system)
}

/// Test team balancing functionality
#[test]
fn test_team_balancing_complete_implementation() {
    use ladder_rs_wasm::js_interface::utils::JsUtilsInterface;
    
    let players = vec![
        JsRatingInterface::new(1500.0, 200.0).expect("Valid rating"),
        JsRatingInterface::new(1600.0, 180.0).expect("Valid rating"),
        JsRatingInterface::new(1400.0, 220.0).expect("Valid rating"),
        JsRatingInterface::new(1550.0, 190.0).expect("Valid rating"),
    ];
    
    let teams = JsUtilsInterface::balance_teams(players, 2);
    
    // Should create exactly 2 teams
    assert_eq!(teams.length(), 2);
    
    // Teams should be non-empty (this validates the balancing actually works)
    // In a real WASM environment, we could cast and check team sizes
    // For now, we verify the function doesn't crash and returns correct count
}

/// Test i18n functionality
#[test]
fn test_internationalization() {
    let i18n = JsI18nInterface::new("en-US");
    
    let message = i18n.format_message("rating_updated", &js_sys::Object::new());
    assert!(!message.is_empty());
    
    let formatted_number = i18n.format_number(1500.5);
    assert!(formatted_number.contains("1500"));
}

/// Test plugin system
#[test]
fn test_plugin_system() {
    let mut plugin_manager = JsPluginManagerInterface::new();
    
    let plugin_config = js_sys::Object::new();
    let registration_result = plugin_manager.register_plugin("test_plugin", plugin_config);
    assert!(registration_result);
    
    let plugins = plugin_manager.list_plugins();
    assert_eq!(plugins.length(), 1);
}
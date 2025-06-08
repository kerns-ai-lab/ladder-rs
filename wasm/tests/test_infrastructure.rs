//! Tests for the test infrastructure itself
//!
//! This module ensures that all test utilities work correctly.

use wasm_bindgen_test::*;
use ladder_rs_wasm::test_utils::*;
use js_sys::Array;
use wasm_bindgen::JsValue;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_fixture_creation() {
    let fixture = TestFixture::new();
    assert_eq!(fixture.player_count(), 0);
    assert_eq!(fixture.match_count(), 0);
}

#[wasm_bindgen_test]
fn test_fixture_player_management() {
    let mut fixture = TestFixture::new();
    
    // Add single player
    let result = fixture.add_test_player("alice").unwrap();
    assert_eq!(result.as_string().unwrap(), "alice");
    assert_eq!(fixture.player_count(), 1);
    
    // Add multiple players
    let players = fixture.add_test_players(5).unwrap();
    assert_eq!(players.length(), 5);
    assert_eq!(fixture.player_count(), 6); // 1 + 5
}

#[wasm_bindgen_test]
fn test_fixture_with_rating_system() {
    let mut fixture = TestFixture::new();
    
    // Setup rating system
    fixture.setup_rating_system("elo").unwrap();
    
    // Add players
    fixture.add_test_player("alice").unwrap();
    fixture.add_test_player("bob").unwrap();
    
    // Simulate match
    let match_id = fixture.simulate_match("alice", "bob", 1).unwrap();
    assert!(match_id.is_string());
    assert_eq!(fixture.match_count(), 1);
}

#[wasm_bindgen_test]
fn test_performance_timer() {
    let mut timer = PerformanceTimer::new();
    
    // Initial state
    assert!(timer.elapsed() >= 0.0);
    
    // Record laps
    let lap1 = timer.lap("first");
    assert!(lap1 >= 0.0);
    
    // Small delay
    let start = js_sys::Date::now();
    while js_sys::Date::now() - start < 10.0 {}
    
    let lap2 = timer.lap("second");
    assert!(lap2 > lap1);
    
    // Get laps
    let laps = timer.get_laps().unwrap();
    assert!(js_sys::Reflect::has(&laps, &JsValue::from_str("first")).unwrap());
    assert!(js_sys::Reflect::has(&laps, &JsValue::from_str("second")).unwrap());
}

#[wasm_bindgen_test]
fn test_mock_data_generator() {
    let mut generator = MockDataGenerator::new(12345);
    
    // Generate IDs
    let id1 = generator.generate_player_id();
    let id2 = generator.generate_player_id();
    assert_ne!(id1, id2);
    assert!(id1.starts_with("player_"));
    
    // Generate names
    let name = generator.generate_player_name();
    assert!(name.contains(' '));
    
    // Generate email
    let email = generator.generate_email();
    assert!(email.contains('@'));
    assert!(email.ends_with("@example.com"));
    
    // Generate outcomes
    let mut outcomes = vec![0u32; 3];
    for _ in 0..100 {
        let outcome = generator.generate_match_outcome();
        assert!(outcome <= 2);
        outcomes[outcome as usize] += 1;
    }
    // Should have some of each outcome
    assert!(outcomes.iter().all(|&count| count > 0));
}

#[wasm_bindgen_test]
fn test_mock_data_generator_batch() {
    let mut generator = MockDataGenerator::new(54321);
    
    let players = generator.generate_players(10);
    assert_eq!(players.length(), 10);
    
    // Check first player structure
    let player = players.get(0);
    assert!(js_sys::Reflect::has(&player, &JsValue::from_str("id")).unwrap());
    assert!(js_sys::Reflect::has(&player, &JsValue::from_str("name")).unwrap());
    assert!(js_sys::Reflect::has(&player, &JsValue::from_str("email")).unwrap());
}

#[wasm_bindgen_test]
fn test_logger() {
    let mut logger = TestLogger::new();
    
    // Log messages
    logger.debug("Debug message");
    logger.info("Info message");
    logger.warn("Warning message");
    logger.error("Error message");
    
    // Check logs
    let logs = logger.get_logs();
    assert_eq!(logs.length(), 4);
    
    // Check contains
    assert!(logger.contains("Debug"));
    assert!(logger.contains("Info"));
    assert!(logger.contains("Warning"));
    assert!(logger.contains("Error"));
    
    // Check counts
    assert_eq!(logger.count_by_level("debug"), 1);
    assert_eq!(logger.count_by_level("info"), 1);
    assert_eq!(logger.count_by_level("warn"), 1);
    assert_eq!(logger.count_by_level("error"), 1);
    
    // Clear logs
    logger.clear();
    assert_eq!(logger.get_logs().length(), 0);
}

#[wasm_bindgen_test]
fn test_logger_enable_disable() {
    let mut logger = TestLogger::new();
    
    logger.info("First message");
    logger.set_enabled(false);
    logger.info("Second message");
    logger.set_enabled(true);
    logger.info("Third message");
    
    assert_eq!(logger.count_by_level("info"), 2); // Only first and third
}

#[wasm_bindgen_test]
fn test_assertion_helper() {
    // Test equals
    AssertionHelper::assert_equals(&JsValue::from_str("abc"), &JsValue::from_str("abc"), "strings").unwrap();
    AssertionHelper::assert_equals(&JsValue::from_f64(42.0), &JsValue::from_f64(42.0), "numbers").unwrap();
    
    // Test truthy/falsy
    AssertionHelper::assert_truthy(&JsValue::from_bool(true), "true").unwrap();
    AssertionHelper::assert_truthy(&JsValue::from_str("hello"), "string").unwrap();
    AssertionHelper::assert_falsy(&JsValue::from_bool(false), "false").unwrap();
    AssertionHelper::assert_falsy(&JsValue::NULL, "null").unwrap();
    
    // Test contains
    let array = Array::new();
    array.push(&JsValue::from_str("apple"));
    array.push(&JsValue::from_str("banana"));
    AssertionHelper::assert_contains(&array, &JsValue::from_str("apple"), "array").unwrap();
    
    // Test range
    AssertionHelper::assert_in_range(5.0, 0.0, 10.0, "in range").unwrap();
}

#[wasm_bindgen_test]
fn test_assertion_helper_failures() {
    // Test equals failure
    let result = AssertionHelper::assert_equals(&JsValue::from_str("abc"), &JsValue::from_str("def"), "should fail");
    assert!(result.is_err());
    
    // Test contains failure
    let array = Array::new();
    array.push(&JsValue::from_str("apple"));
    let result = AssertionHelper::assert_contains(&array, &JsValue::from_str("orange"), "should fail");
    assert!(result.is_err());
    
    // Test range failure
    let result = AssertionHelper::assert_in_range(15.0, 0.0, 10.0, "should fail");
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_snapshot() {
    let snapshot1 = TestSnapshot::new("test data v1");
    let snapshot2 = TestSnapshot::new("test data v1");
    let snapshot3 = TestSnapshot::new("test data v2");
    
    // Test equality
    assert!(snapshot1.equals(&snapshot2));
    assert!(!snapshot1.equals(&snapshot3));
    
    // Test data access
    assert_eq!(snapshot1.get_data(), "test data v1");
    
    // Test timestamp
    assert!(snapshot1.get_timestamp() > 0.0);
    
    // Test diff
    let diff = snapshot1.diff(&snapshot2);
    assert_eq!(diff, "No differences");
    
    let diff = snapshot1.diff(&snapshot3);
    assert!(diff.contains("Snapshots differ"));
}

#[wasm_bindgen_test]
fn test_browser_environment() {
    // These tests will have different results in browser vs Node
    let is_browser = BrowserEnvironment::is_browser();
    let is_node = BrowserEnvironment::is_node();
    
    // One should be true, the other false
    assert_ne!(is_browser, is_node);
    
    // User agent might be available
    let user_agent = BrowserEnvironment::get_user_agent();
    if is_browser {
        assert!(user_agent.is_some());
    }
    
    // Check feature detection
    let has_local_storage = BrowserEnvironment::has_local_storage();
    let has_web_workers = BrowserEnvironment::has_web_workers();
    
    // In browser, these might be available
    if is_browser {
        // Just verify the methods work without crashing
        let _ = has_local_storage;
        let _ = has_web_workers;
    }
}
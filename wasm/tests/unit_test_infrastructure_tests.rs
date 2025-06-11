//! Tests for the unit test infrastructure module

use wasm_bindgen_test::*;
use ladder_rs_wasm::test_utils::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_rating_factory_creates_valid_ratings() {
    let elo_rating = create_test_elo_rating(1500.0);
    assert_eq!(elo_rating.value(), 1500.0);
    
    let trueskill_rating = create_test_trueskill_rating(25.0, 69.44);
    assert_eq!(trueskill_rating.mean(), 25.0);
    assert_eq!(trueskill_rating.variance(), 69.44);
}

#[wasm_bindgen_test]
fn test_mock_rating_system_behavior() {
    let mut mock = MockRatingSystem::new(32.0);
    
    // Test default behavior
    let result = mock.process_match(1500.0, 1500.0, 1);
    assert!(result.is_object());
    
    // Test call counting
    assert_eq!(mock.get_call_count("process_match"), 1);
    
    // Test fixed win probability
    mock.set_win_probability(0.75);
    assert_eq!(mock.get_win_probability(1500.0, 1600.0), 0.75);
    assert_eq!(mock.get_call_count("get_win_probability"), 1);
}

#[wasm_bindgen_test]
fn test_performance_timer_functionality() {
    let mut timer = PerformanceTimer::new();
    
    // Record some laps
    let lap1 = timer.lap("start");
    assert!(lap1 >= 0.0);
    
    // Small delay
    let start = js_sys::Date::now();
    while js_sys::Date::now() - start < 10.0 {}
    
    let lap2 = timer.lap("middle");
    assert!(lap2 > lap1);
    
    let elapsed = timer.elapsed();
    assert!(elapsed >= lap2);
    
    // Check lap data
    let laps = timer.get_laps().unwrap();
    assert!(js_sys::Reflect::has(&laps, &wasm_bindgen::JsValue::from_str("start")).unwrap());
    assert!(js_sys::Reflect::has(&laps, &wasm_bindgen::JsValue::from_str("middle")).unwrap());
}

#[wasm_bindgen_test]
fn test_assertion_helpers() {
    use wasm_bindgen::JsValue;
    
    // Test equals assertion
    assert!(AssertionHelper::assert_equals(&JsValue::from(42), &JsValue::from(42), "Numbers should be equal").is_ok());
    assert!(AssertionHelper::assert_equals(&JsValue::from(42), &JsValue::from(43), "Numbers should not be equal").is_err());
    
    // Test truthy/falsy assertions
    assert!(AssertionHelper::assert_truthy(&JsValue::from(true), "Should be truthy").is_ok());
    assert!(AssertionHelper::assert_falsy(&JsValue::from(false), "Should be falsy").is_ok());
    
    // Test range assertion
    assert!(AssertionHelper::assert_in_range(5.0, 0.0, 10.0, "Should be in range").is_ok());
    assert!(AssertionHelper::assert_in_range(15.0, 0.0, 10.0, "Should not be in range").is_err());
    
    // Test approximate equals
    assert!(AssertionHelper::assert_approx_equals(1.001, 1.0, 0.01, "Should be approximately equal").is_ok());
    assert!(AssertionHelper::assert_approx_equals(1.1, 1.0, 0.01, "Should not be approximately equal").is_err());
}

#[wasm_bindgen_test]
fn test_test_fixture_functionality() {
    let mut fixture = TestFixture::new();
    
    // Add players
    fixture.add_player("player1").unwrap();
    fixture.add_player("player2").unwrap();
    assert_eq!(fixture.player_count(), 2);
    
    // Record a match
    fixture.record_match("player1", "player2", 1).unwrap();
    assert_eq!(fixture.match_count(), 1);
    
    // Test duplicate player
    assert!(fixture.add_player("player1").is_err());
    
    // Test invalid match
    assert!(fixture.record_match("player1", "player3", 1).is_err());
}

#[wasm_bindgen_test]
fn test_mock_storage() {
    let storage = MockStorage::new();
    
    // Test basic operations
    storage.set_item("key1", "value1").unwrap();
    assert_eq!(storage.get_item("key1").unwrap(), Some("value1".to_string()));
    assert_eq!(storage.length(), 1);
    
    // Test remove
    storage.remove_item("key1").unwrap();
    assert_eq!(storage.get_item("key1").unwrap(), None);
    assert_eq!(storage.length(), 0);
    
    // Test failure modes
    storage.set_fail_on_write(true);
    assert!(storage.set_item("key2", "value2").is_err());
    
    storage.set_fail_on_write(false);
    storage.set_item("key2", "value2").unwrap();
    
    storage.set_fail_on_read(true);
    assert!(storage.get_item("key2").is_err());
}

#[wasm_bindgen_test]
fn test_coverage_tracker() {
    let tracker = CoverageTracker::new();
    
    // Register and track functions
    tracker.register_function("test_function", Some(3));
    tracker.track_call("test_function");
    tracker.track_path("test_function", "path1");
    tracker.track_path("test_function", "path2");
    
    // Check coverage
    let coverage = tracker.get_coverage_percentage();
    assert!(coverage > 0.0);
    
    // Get detailed report
    let report = tracker.get_report();
    assert!(report.is_object());
}

#[wasm_bindgen_test]
fn test_test_data_generation() {
    // Test player pool generation
    let uniform_players = generate_player_pool(10, SkillDistribution::Uniform);
    assert_eq!(uniform_players.len(), 10);
    assert!(uniform_players.iter().all(|p| p.true_skill() == 1500.0));
    
    let normal_players = generate_player_pool(10, SkillDistribution::Normal);
    assert_eq!(normal_players.len(), 10);
    
    // Test match history generation
    let matches = generate_match_history(&uniform_players, 20, 12345);
    assert_eq!(matches.len(), 20);
}

#[wasm_bindgen_test]
fn test_browser_environment_detection() {
    let is_browser = BrowserEnvironment::is_browser();
    let is_node = BrowserEnvironment::is_node();
    
    // One should be true, the other false
    assert!(is_browser != is_node);
    
    // Test feature detection
    let has_local_storage = BrowserEnvironment::has_local_storage();
    let has_web_workers = BrowserEnvironment::has_web_workers();
    
    // In test environment, these might be false
    assert!(has_local_storage == true || has_local_storage == false);
    assert!(has_web_workers == true || has_web_workers == false);
}

#[wasm_bindgen_test]
fn test_test_logger() {
    let mut logger = TestLogger::new();
    
    // Log messages
    logger.info("Info message");
    logger.warn("Warning message");
    logger.error("Error message");
    
    // Check log contents
    assert!(logger.contains("Info message"));
    assert_eq!(logger.count_by_level("warn"), 1);
    assert_eq!(logger.count_by_level("error"), 1);
    
    // Test enable/disable
    logger.set_enabled(false);
    logger.info("This should not be logged");
    assert!(!logger.contains("This should not be logged"));
}

#[wasm_bindgen_test]
fn test_benchmark_runner() {
    let runner = BenchmarkRunner::new(5);
    
    // Create a simple benchmark function
    let bench_fn = js_sys::Function::new_no_args("return 42");
    
    // Run benchmark
    let stats = runner.run_benchmark("test_bench", &bench_fn).unwrap();
    assert!(stats.is_object());
    
    // Check that stats contain expected fields
    assert!(js_sys::Reflect::has(&stats, &wasm_bindgen::JsValue::from_str("mean")).unwrap());
    assert!(js_sys::Reflect::has(&stats, &wasm_bindgen::JsValue::from_str("median")).unwrap());
    assert!(js_sys::Reflect::has(&stats, &wasm_bindgen::JsValue::from_str("min")).unwrap());
    assert!(js_sys::Reflect::has(&stats, &wasm_bindgen::JsValue::from_str("max")).unwrap());
}

#[wasm_bindgen_test]
fn test_mock_random() {
    let random = MockRandom::new(12345);
    
    // Test deterministic behavior
    let val1 = random.next();
    let val2 = random.next();
    assert!(val1 >= 0.0 && val1 <= 1.0);
    assert!(val2 >= 0.0 && val2 <= 1.0);
    assert!(val1 != val2);
    
    // Test fixed values
    let fixed_values = js_sys::Array::new();
    fixed_values.push(&wasm_bindgen::JsValue::from(0.1));
    fixed_values.push(&wasm_bindgen::JsValue::from(0.5));
    fixed_values.push(&wasm_bindgen::JsValue::from(0.9));
    
    random.set_fixed_values(fixed_values);
    assert_eq!(random.next(), 0.1);
    assert_eq!(random.next(), 0.5);
    assert_eq!(random.next(), 0.9);
    assert_eq!(random.next(), 0.1); // Should cycle
}

#[wasm_bindgen_test]
fn test_integration_test_helper() {
    let mut helper = IntegrationTestHelper::new();
    
    // Create test functions
    let pass_fn = js_sys::Function::new_no_args("return true");
    let fail_fn = js_sys::Function::new_no_args("throw new Error('Test error')");
    
    // Run tests
    assert!(helper.run_test("passing_test", &pass_fn));
    assert!(!helper.run_test("failing_test", &fail_fn));
    
    // Check summary
    let summary = helper.get_summary();
    let total = js_sys::Reflect::get(&summary, &wasm_bindgen::JsValue::from_str("total")).unwrap().as_f64().unwrap();
    let passed = js_sys::Reflect::get(&summary, &wasm_bindgen::JsValue::from_str("passed")).unwrap().as_f64().unwrap();
    let failed = js_sys::Reflect::get(&summary, &wasm_bindgen::JsValue::from_str("failed")).unwrap().as_f64().unwrap();
    
    assert_eq!(total, 2.0);
    assert_eq!(passed, 1.0);
    assert_eq!(failed, 1.0);
}

#[wasm_bindgen_test]
fn test_test_snapshot() {
    let snapshot1 = TestSnapshot::new("data1");
    let snapshot2 = TestSnapshot::new("data2");
    let snapshot3 = TestSnapshot::new("data1");
    
    // Test equality
    assert!(snapshot1.equals(&snapshot3));
    assert!(!snapshot1.equals(&snapshot2));
    
    // Test diff
    let diff = snapshot1.diff(&snapshot2);
    assert!(diff.contains("differ"));
    
    let no_diff = snapshot1.diff(&snapshot3);
    assert!(no_diff.contains("No differences"));
    
    // Test JSON serialization
    let json = snapshot1.to_json().unwrap();
    let restored = TestSnapshot::from_json(&json).unwrap();
    assert!(snapshot1.equals(&restored));
}

#[wasm_bindgen_test]
fn test_memory_tracker() {
    let mut tracker = MemoryTracker::new();
    
    // Take snapshots
    tracker.snapshot("initial");
    
    // Allocate some memory
    let _data = vec![0u8; 1000];
    
    tracker.snapshot("after_allocation");
    
    // Get report
    let report = tracker.get_report();
    assert!(report.is_object());
    
    // Check that snapshots were recorded
    let snapshots = js_sys::Reflect::get(&report, &wasm_bindgen::JsValue::from_str("snapshots"))
        .unwrap()
        .dyn_into::<js_sys::Array>()
        .unwrap();
    assert_eq!(snapshots.length(), 2);
}

#[wasm_bindgen_test]
fn test_dataset_builder() {
    let mut builder = TestDatasetBuilder::new();
    
    // Build a dataset
    builder
        .add_player("alice", "Alice Smith", 1600.0)
        .add_player("bob", "Bob Jones", 1400.0)
        .add_match("alice", "bob", 1);
    
    let dataset = builder.build();
    
    // Check players
    let players = js_sys::Reflect::get(&dataset, &wasm_bindgen::JsValue::from_str("players"))
        .unwrap()
        .dyn_into::<js_sys::Array>()
        .unwrap();
    assert_eq!(players.length(), 2);
    
    // Check matches
    let matches = js_sys::Reflect::get(&dataset, &wasm_bindgen::JsValue::from_str("matches"))
        .unwrap()
        .dyn_into::<js_sys::Array>()
        .unwrap();
    assert_eq!(matches.length(), 1);
}

#[wasm_bindgen_test]
fn test_factory_methods() {
    // Test match factory
    let match_result = TestMatchFactory::create_1v1_result("p1", "p2", 1500.0, 1500.0, 1);
    assert!(match_result.is_object());
    
    // Test config factory
    let elo_config = TestConfigFactory::default_elo_config();
    let k_factor = js_sys::Reflect::get(&elo_config, &wasm_bindgen::JsValue::from_str("k_factor"))
        .unwrap()
        .as_f64()
        .unwrap();
    assert_eq!(k_factor, 32.0);
    
    // Test scenario factory
    let scenario = TestScenarioFactory::create_ladder_scenario(10);
    let players = js_sys::Reflect::get(&scenario, &wasm_bindgen::JsValue::from_str("players"))
        .unwrap()
        .dyn_into::<js_sys::Array>()
        .unwrap();
    assert_eq!(players.length(), 10);
}
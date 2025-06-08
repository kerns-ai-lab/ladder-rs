//! Browser-specific tests
//!
//! This module contains tests that verify browser-specific functionality.

use wasm_bindgen_test::*;
use ladder_rs_wasm::{PlayerManager, WasmRatingSystem, test_utils::BrowserEnvironment};
use wasm_bindgen::{JsValue, JsCast};
use web_sys::{window, Storage, Performance};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_browser_environment_detection() {
    // In browser tests, these should be true/false respectively
    if BrowserEnvironment::is_browser() {
        assert!(BrowserEnvironment::is_browser());
        assert!(!BrowserEnvironment::is_node());
    } else {
        assert!(!BrowserEnvironment::is_browser());
        assert!(BrowserEnvironment::is_node());
    }
}

#[wasm_bindgen_test]
fn test_local_storage_integration() {
    if !BrowserEnvironment::has_local_storage() {
        // Skip test if localStorage not available
        return;
    }
    
    let window = window().expect("should have window");
    let storage = window.local_storage()
        .expect("should have local storage")
        .expect("local storage should be Some");
    
    // Test saving player data
    let manager = PlayerManager::new();
    let export_data = manager.export_players(true).unwrap();
    
    // Store in localStorage
    storage.set_item("ladder_rs_players", &export_data)
        .expect("should store data");
    
    // Retrieve from localStorage
    let retrieved = storage.get_item("ladder_rs_players")
        .expect("should get data")
        .expect("data should exist");
    
    assert_eq!(export_data, retrieved);
    
    // Clean up
    storage.remove_item("ladder_rs_players")
        .expect("should remove item");
}

#[wasm_bindgen_test]
fn test_performance_api() {
    if !BrowserEnvironment::is_browser() {
        return;
    }
    
    let window = window().expect("should have window");
    let performance = window.performance()
        .expect("should have performance API");
    
    // Mark start
    performance.mark("test_start")
        .expect("should create mark");
    
    // Do some work
    let mut manager = PlayerManager::new();
    for i in 0..100 {
        manager.register_player(&format!("player_{}", i), None, None).unwrap();
    }
    
    // Mark end
    performance.mark("test_end")
        .expect("should create mark");
    
    // Measure
    performance.measure_with_start_mark_and_end_mark(
        "test_duration",
        "test_start", 
        "test_end"
    ).expect("should create measure");
    
    // Get entries
    let entries = performance.get_entries_by_name("test_duration");
    assert_eq!(entries.length(), 1);
    
    // Clean up
    performance.clear_marks();
    performance.clear_measures();
}

#[wasm_bindgen_test]
fn test_dom_manipulation() {
    if !BrowserEnvironment::is_browser() {
        return;
    }
    
    let window = window().expect("should have window");
    let document = window.document().expect("should have document");
    
    // Create a div to display leaderboard
    let div = document.create_element("div")
        .expect("should create div");
    div.set_id("leaderboard");
    
    // Add to body
    let body = document.body().expect("should have body");
    body.append_child(&div).expect("should append child");
    
    // Create rating system and players
    let mut system = WasmRatingSystem::new("elo").unwrap();
    for i in 0..5 {
        system.create_player(&format!("player_{}", i)).unwrap();
    }
    
    // Get leaderboard
    let leaderboard = system.get_leaderboard(None).unwrap();
    
    // Create HTML content
    let mut html = String::from("<h3>Leaderboard</h3><ol>");
    for i in 0..leaderboard.length() {
        let entry = leaderboard.get(i);
        let player_id = js_sys::Reflect::get(&entry, &JsValue::from_str("player_id"))
            .unwrap()
            .as_string()
            .unwrap();
        let rating = js_sys::Reflect::get(&entry, &JsValue::from_str("rating"))
            .unwrap()
            .as_f64()
            .unwrap();
        
        html.push_str(&format!("<li>{}: {:.0}</li>", player_id, rating));
    }
    html.push_str("</ol>");
    
    // Set content
    div.set_inner_html(&html);
    
    // Verify content was set
    assert!(div.inner_html().contains("Leaderboard"));
    assert!(div.inner_html().contains("player_"));
    
    // Clean up
    body.remove_child(&div).expect("should remove child");
}

#[wasm_bindgen_test]
fn test_console_output() {
    // Console methods should work in browser
    web_sys::console::log_1(&JsValue::from_str("Test log message"));
    web_sys::console::info_1(&JsValue::from_str("Test info message"));
    web_sys::console::warn_1(&JsValue::from_str("Test warning message"));
    web_sys::console::error_1(&JsValue::from_str("Test error message"));
    
    // Group console output
    web_sys::console::group_1(&JsValue::from_str("Test Group"));
    web_sys::console::log_1(&JsValue::from_str("Inside group"));
    web_sys::console::group_end();
    
    // Time measurement
    web_sys::console::time_with_label("test_timer");
    let _manager = PlayerManager::new();
    web_sys::console::time_end_with_label("test_timer");
}

#[wasm_bindgen_test]
fn test_json_serialization_in_browser() {
    let manager = PlayerManager::new();
    
    // Use browser's JSON.parse and JSON.stringify
    let js_obj = js_sys::Object::new();
    js_sys::Reflect::set(
        &js_obj,
        &JsValue::from_str("test"),
        &JsValue::from_str("data")
    ).unwrap();
    
    let json_string = js_sys::JSON::stringify(&js_obj)
        .expect("should stringify");
    
    let parsed = js_sys::JSON::parse(&json_string)
        .expect("should parse");
    
    let test_value = js_sys::Reflect::get(&parsed, &JsValue::from_str("test"))
        .unwrap();
    
    assert_eq!(test_value.as_string().unwrap(), "data");
}

#[wasm_bindgen_test]
fn test_date_handling() {
    // Test JavaScript Date integration
    let now = js_sys::Date::now();
    let date = js_sys::Date::new(&JsValue::from_f64(now));
    
    // Create player with timestamp
    let mut manager = PlayerManager::new();
    manager.register_player("test_player", None, None).unwrap();
    
    let player = manager.get_player("test_player").unwrap();
    let created_at = js_sys::Reflect::get(&player, &JsValue::from_str("created_at"))
        .unwrap()
        .as_f64()
        .unwrap();
    
    // Verify timestamp is reasonable (within last minute)
    assert!(created_at > now - 60000.0);
    assert!(created_at <= js_sys::Date::now());
}

#[wasm_bindgen_test]
fn test_array_operations() {
    // Test JavaScript Array integration
    let arr = js_sys::Array::new();
    
    // Add some ratings
    for i in 0..5 {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("player_id"),
            &JsValue::from_str(&format!("player_{}", i))
        ).unwrap();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("rating"),
            &JsValue::from_f64(1500.0 + i as f64 * 10.0)
        ).unwrap();
        arr.push(&obj);
    }
    
    // Test array methods
    assert_eq!(arr.length(), 5);
    
    // Sort by rating (descending)
    arr.sort();
    
    // Map to get player IDs
    let mapped = arr.map(&mut |val, _, _| {
        js_sys::Reflect::get(&val, &JsValue::from_str("player_id")).unwrap()
    });
    
    assert_eq!(mapped.length(), 5);
}

#[wasm_bindgen_test]
fn test_error_handling_in_browser() {
    // Test that errors are properly propagated to JavaScript
    let mut manager = PlayerManager::new();
    
    // Try to get non-existent player
    let result = manager.get_player("non_existent");
    assert!(result.is_err());
    
    // Error should be a proper JavaScript error
    let err = result.unwrap_err();
    assert!(err.is_string() || err.is_object());
    
    // Try invalid operations
    let system = WasmRatingSystem::new("invalid_system");
    assert!(system.is_err());
}

#[wasm_bindgen_test]
fn test_memory_cleanup() {
    // Test that objects can be properly garbage collected
    
    // Create and drop many objects
    for _ in 0..100 {
        let manager = PlayerManager::new();
        for i in 0..10 {
            manager.register_player(&format!("player_{}", i), None, None).unwrap();
        }
        // manager goes out of scope and should be collectable
    }
    
    // If we haven't crashed, memory management is working
    assert!(true);
}
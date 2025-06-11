//! Cross-browser compatibility tests for the WASM module
//! 
//! This test suite verifies that the WASM module works correctly across
//! different browser environments and JavaScript engines.

use wasm_bindgen_test::*;
use wasm_bindgen::prelude::*;
use web_sys::{window, console};
use js_sys::{Array, Date, Function, Object, Promise, Reflect};
use ladder_rs_wasm::{CrossBrowserCompat, EventCompat, PerformanceCompat};

wasm_bindgen_test_configure!(run_in_browser);

// Browser detection and identification tests
#[wasm_bindgen_test]
fn test_browser_detection() {
    let browser = CrossBrowserCompat::get_browser_info();
    assert!(browser.is_some());
    
    let browser_name = browser.unwrap();
    console::log_1(&format!("Detected browser: {}", browser_name).into());
    
    // Verify it's one of the known browsers
    assert!(
        ["Chrome", "Firefox", "Safari", "Edge", "Opera", "Unknown"].contains(&browser_name.as_str()),
        "Browser should be one of the known types"
    );
}

// Feature detection tests
#[wasm_bindgen_test]
fn test_feature_detection() {
    // Core features that should be available in all modern browsers
    assert!(CrossBrowserCompat::has_feature("promise"), "Promise should be available");
    assert!(CrossBrowserCompat::has_feature("webAssembly"), "WebAssembly should be available");
    
    // Storage features
    let has_local_storage = CrossBrowserCompat::has_feature("localStorage");
    let has_session_storage = CrossBrowserCompat::has_feature("sessionStorage");
    console::log_1(&format!("LocalStorage: {}, SessionStorage: {}", has_local_storage, has_session_storage).into());
    
    // Advanced features (may not be available in all environments)
    let has_workers = CrossBrowserCompat::has_feature("webWorker");
    let has_service_worker = CrossBrowserCompat::has_feature("serviceWorker");
    let has_indexed_db = CrossBrowserCompat::has_feature("indexedDB");
    
    console::log_1(&format!(
        "Advanced features - Workers: {}, ServiceWorker: {}, IndexedDB: {}",
        has_workers, has_service_worker, has_indexed_db
    ).into());
}

// Console API compatibility tests
#[wasm_bindgen_test]
fn test_console_compatibility() {
    // These should not panic even if console is not available
    CrossBrowserCompat::console_log("Test log message");
    CrossBrowserCompat::console_error("Test error message");
    CrossBrowserCompat::console_warn("Test warning message");
    
    // Direct console access should also work
    console::log_1(&"Direct console log".into());
    console::error_1(&"Direct console error".into());
    console::warn_1(&"Direct console warn".into());
}

// Storage API compatibility tests
#[wasm_bindgen_test]
fn test_storage_compatibility() {
    if let Some(storage) = CrossBrowserCompat::get_safe_storage() {
        let test_key = "ladder_rs_compat_test";
        let test_value = "test_value_12345";
        
        // Test basic storage operations
        storage.set_item(test_key, test_value).expect("Should be able to set item");
        
        let retrieved = storage.get_item(test_key).expect("Should be able to get item");
        assert_eq!(retrieved, Some(test_value.to_string()));
        
        storage.remove_item(test_key).expect("Should be able to remove item");
        
        let after_remove = storage.get_item(test_key).expect("Should be able to check after removal");
        assert!(after_remove.is_none());
    } else {
        console::warn_1(&"No storage available in this environment".into());
    }
}

// Performance API compatibility tests
#[wasm_bindgen_test]
fn test_performance_compatibility() {
    // Test performance.now() with fallback
    let time1 = PerformanceCompat::now();
    let time2 = PerformanceCompat::now();
    
    assert!(time2 >= time1, "Time should be monotonically increasing");
    assert!(time1 > 0.0, "Time should be positive");
    
    // Test performance marks and measures
    PerformanceCompat::mark("test_start");
    
    // Do some work
    let mut sum = 0;
    for i in 0..1000 {
        sum += i;
    }
    
    PerformanceCompat::mark("test_end");
    PerformanceCompat::measure("test_duration", "test_start", "test_end");
    
    console::log_1(&format!("Performance test sum: {}", sum).into());
}

// Event handling compatibility tests
#[wasm_bindgen_test]
fn test_event_compatibility() {
    if let Some(window) = window() {
        let document = window.document().expect("Should have document");
        
        // Create a test element
        let element = document.create_element("div").expect("Should create element");
        element.set_id("test_event_element");
        
        // Test event listener
        let handler = Function::new_no_args("console.log('Event fired')");
        
        EventCompat::add_listener(&element, "click", &handler)
            .expect("Should add event listener");
        
        EventCompat::remove_listener(&element, "click", &handler)
            .expect("Should remove event listener");
        
        // Test custom event
        let detail = JsValue::from_str("test_detail");
        let custom_event = EventCompat::create_custom_event("test_custom", &detail)
            .expect("Should create custom event");
        
        assert_eq!(custom_event.type_(), "test_custom");
    }
}

// JavaScript type compatibility tests
#[wasm_bindgen_test]
fn test_js_type_compatibility() {
    // Test Date
    let date = Date::new_0();
    assert!(date.get_time() > 0.0);
    
    // Test Array
    let array = Array::new();
    array.push(&JsValue::from(1));
    array.push(&JsValue::from(2));
    array.push(&JsValue::from(3));
    assert_eq!(array.length(), 3);
    
    // Test Object
    let obj = Object::new();
    Reflect::set(&obj, &"key".into(), &"value".into()).expect("Should set property");
    
    let value = Reflect::get(&obj, &"key".into()).expect("Should get property");
    assert_eq!(value.as_string(), Some("value".to_string()));
    
    // Test Promise (if available)
    if CrossBrowserCompat::has_feature("promise") {
        let promise = Promise::resolve(&JsValue::from(42));
        assert!(promise.is_instance_of::<Promise>());
    }
}

// Typed array compatibility tests
#[wasm_bindgen_test]
fn test_typed_array_compatibility() {
    use js_sys::{Float32Array, Uint8Array, Uint32Array};
    
    // Test Float32Array
    let float_array = Float32Array::new_with_length(10);
    float_array.set_index(0, 1.5);
    float_array.set_index(1, 2.5);
    assert_eq!(float_array.get_index(0), 1.5);
    assert_eq!(float_array.get_index(1), 2.5);
    
    // Test Uint8Array
    let bytes = vec![1u8, 2, 3, 4, 5];
    let uint8_array = Uint8Array::from(&bytes[..]);
    assert_eq!(uint8_array.length(), 5);
    assert_eq!(uint8_array.get_index(2), 3);
    
    // Test Uint32Array
    let uint32_array = Uint32Array::new_with_length(3);
    uint32_array.set_index(0, 100);
    uint32_array.set_index(1, 200);
    uint32_array.set_index(2, 300);
    assert_eq!(uint32_array.get_index(1), 200);
}

// WebAssembly API compatibility tests
#[wasm_bindgen_test]
fn test_webassembly_compatibility() {
    use js_sys::WebAssembly;
    
    // Test WebAssembly object exists
    assert!(CrossBrowserCompat::has_feature("webAssembly"));
    
    // Test WebAssembly.Memory
    let memory = WebAssembly::Memory::new(&JsValue::from(
        Object::from_entries(&Array::from(&JsValue::from(
            vec![
                Array::from(&JsValue::from(vec!["initial", 1])),
                Array::from(&JsValue::from(vec!["maximum", 10])),
            ]
        ))).expect("Should create object")
    )).expect("Should create memory");
    
    // Memory should be created successfully
    let buffer = memory.buffer();
    assert!(buffer.byte_length() > 0);
}

// Math functions compatibility tests
#[wasm_bindgen_test]
fn test_math_compatibility() {
    use js_sys::Math;
    
    // Test basic math functions
    assert_eq!(Math::abs(-5.0), 5.0);
    assert_eq!(Math::ceil(4.2), 5.0);
    assert_eq!(Math::floor(4.8), 4.0);
    assert_eq!(Math::round(4.5), 5.0);
    
    // Test trigonometric functions
    assert!((Math::sin(0.0) - 0.0).abs() < 0.0001);
    assert!((Math::cos(0.0) - 1.0).abs() < 0.0001);
    
    // Test min/max
    assert_eq!(Math::min(5.0, 3.0), 3.0);
    assert_eq!(Math::max(5.0, 3.0), 5.0);
    
    // Test random (should be between 0 and 1)
    let random = Math::random();
    assert!(random >= 0.0 && random < 1.0);
}

// JSON compatibility tests
#[wasm_bindgen_test]
fn test_json_compatibility() {
    use js_sys::JSON;
    
    // Test JSON stringify
    let obj = Object::new();
    Reflect::set(&obj, &"name".into(), &"test".into()).unwrap();
    Reflect::set(&obj, &"value".into(), &42.into()).unwrap();
    
    let json_string = JSON::stringify(&obj).expect("Should stringify");
    assert!(json_string.as_string().unwrap().contains("\"name\":\"test\""));
    assert!(json_string.as_string().unwrap().contains("\"value\":42"));
    
    // Test JSON parse
    let parsed = JSON::parse(&json_string).expect("Should parse");
    let name = Reflect::get(&parsed, &"name".into()).unwrap();
    let value = Reflect::get(&parsed, &"value".into()).unwrap();
    
    assert_eq!(name.as_string(), Some("test".to_string()));
    assert_eq!(value.as_f64(), Some(42.0));
}

// Error handling compatibility tests
#[wasm_bindgen_test]
fn test_error_compatibility() {
    use js_sys::Error;
    
    // Test creating errors
    let error = Error::new("Test error message");
    assert_eq!(error.message(), "Test error message");
    
    // Test error with cause
    let cause = Error::new("Cause error");
    let error_with_cause = Error::new_with_cause("Main error", &cause);
    assert_eq!(error_with_cause.message(), "Main error");
}

// Comprehensive browser API support test
#[wasm_bindgen_test] 
fn test_browser_api_matrix() {
    let features = vec![
        "localStorage",
        "sessionStorage",
        "indexedDB",
        "webWorker",
        "serviceWorker",
        "webAssembly",
        "performance",
        "promise",
        "fetch",
    ];
    
    console::log_1(&"=== Browser API Support Matrix ===".into());
    
    for feature in features {
        let supported = CrossBrowserCompat::has_feature(feature);
        console::log_1(&format!("{}: {}", feature, if supported { "✅" } else { "❌" }).into());
    }
    
    console::log_1(&"=================================".into());
}

// Helper function for async testing
pub fn detect_browser() -> String {
    CrossBrowserCompat::get_browser_info().unwrap_or_else(|| "Unknown".to_string())
}

// Helper function to check if running in Node.js
pub fn is_node_environment() -> bool {
    if let Some(window) = window() {
        // In browser environment
        false
    } else {
        // Likely in Node.js
        true
    }
}

// Helper function to generate compatibility report
pub fn generate_compatibility_report() -> Object {
    let report = Object::new();
    
    Reflect::set(&report, &"browser".into(), &detect_browser().into()).unwrap();
    Reflect::set(&report, &"timestamp".into(), &Date::new_0().to_iso_string().into()).unwrap();
    Reflect::set(&report, &"is_node".into(), &is_node_environment().into()).unwrap();
    
    let features = Object::new();
    let feature_list = vec![
        "localStorage", "sessionStorage", "indexedDB", 
        "webWorker", "serviceWorker", "webAssembly",
        "performance", "promise", "fetch"
    ];
    
    for feature in feature_list {
        Reflect::set(
            &features, 
            &feature.into(), 
            &CrossBrowserCompat::has_feature(feature).into()
        ).unwrap();
    }
    
    Reflect::set(&report, &"features".into(), &features).unwrap();
    
    report
}
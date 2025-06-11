//! Integration test helpers for browser and environment testing

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Helper for integration testing across different environments
#[wasm_bindgen]
pub struct IntegrationTestHelper {
    test_results: Vec<TestResult>,
}

struct TestResult {
    name: String,
    passed: bool,
    duration_ms: f64,
    error: Option<String>,
}

#[wasm_bindgen]
impl IntegrationTestHelper {
    /// Create a new integration test helper
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            test_results: Vec::new(),
        }
    }

    /// Run a test and record the result
    pub fn run_test(&mut self, name: &str, test_fn: &js_sys::Function) -> bool {
        let start = js_sys::Date::now();
        
        let result = match test_fn.call0(&JsValue::NULL) {
            Ok(_) => TestResult {
                name: name.to_string(),
                passed: true,
                duration_ms: js_sys::Date::now() - start,
                error: None,
            },
            Err(e) => TestResult {
                name: name.to_string(),
                passed: false,
                duration_ms: js_sys::Date::now() - start,
                error: Some(format!("{:?}", e)),
            },
        };

        let passed = result.passed;
        self.test_results.push(result);
        passed
    }

    /// Get test results summary
    pub fn get_summary(&self) -> js_sys::Object {
        let summary = js_sys::Object::new();
        
        let total = self.test_results.len() as f64;
        let passed = self.test_results.iter().filter(|r| r.passed).count() as f64;
        let failed = total - passed;
        let total_duration: f64 = self.test_results.iter().map(|r| r.duration_ms).sum();

        js_sys::Reflect::set(
            &summary,
            &JsValue::from_str("total"),
            &JsValue::from_f64(total),
        ).unwrap();
        js_sys::Reflect::set(
            &summary,
            &JsValue::from_str("passed"),
            &JsValue::from_f64(passed),
        ).unwrap();
        js_sys::Reflect::set(
            &summary,
            &JsValue::from_str("failed"),
            &JsValue::from_f64(failed),
        ).unwrap();
        js_sys::Reflect::set(
            &summary,
            &JsValue::from_str("duration_ms"),
            &JsValue::from_f64(total_duration),
        ).unwrap();

        summary
    }

    /// Get detailed test results
    pub fn get_results(&self) -> js_sys::Array {
        let results = js_sys::Array::new();
        
        for result in &self.test_results {
            let obj = js_sys::Object::new();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("name"),
                &JsValue::from_str(&result.name),
            ).unwrap();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("passed"),
                &JsValue::from_bool(result.passed),
            ).unwrap();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("duration_ms"),
                &JsValue::from_f64(result.duration_ms),
            ).unwrap();
            
            if let Some(ref error) = result.error {
                js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("error"),
                    &JsValue::from_str(error),
                ).unwrap();
            }
            
            results.push(&obj);
        }
        
        results
    }

    /// Clear test results
    pub fn clear(&mut self) {
        self.test_results.clear();
    }
}

/// Browser environment detector
#[wasm_bindgen]
pub struct BrowserEnvironment;

#[wasm_bindgen]
impl BrowserEnvironment {
    /// Check if running in a browser
    pub fn is_browser() -> bool {
        web_sys::window().is_some()
    }

    /// Check if running in Node.js
    pub fn is_node() -> bool {
        !Self::is_browser()
    }

    /// Get user agent string
    pub fn get_user_agent() -> Option<String> {
        web_sys::window()?
            .navigator()
            .user_agent()
            .ok()
    }

    /// Check if localStorage is available
    pub fn has_local_storage() -> bool {
        if let Some(window) = web_sys::window() {
            window.local_storage().ok().flatten().is_some()
        } else {
            false
        }
    }

    /// Check if WebWorkers are available
    pub fn has_web_workers() -> bool {
        if let Some(window) = web_sys::window() {
            let worker = js_sys::Reflect::get(&window, &JsValue::from_str("Worker")).ok();
            worker.is_some() && !worker.unwrap().is_undefined()
        } else {
            false
        }
    }

    /// Check if IndexedDB is available
    pub fn has_indexed_db() -> bool {
        if let Some(window) = web_sys::window() {
            let idb = js_sys::Reflect::get(&window, &JsValue::from_str("indexedDB")).ok();
            idb.is_some() && !idb.unwrap().is_undefined()
        } else {
            false
        }
    }

    /// Get browser info object
    pub fn get_info() -> js_sys::Object {
        let info = js_sys::Object::new();

        js_sys::Reflect::set(
            &info,
            &JsValue::from_str("is_browser"),
            &JsValue::from_bool(Self::is_browser()),
        ).unwrap();
        
        js_sys::Reflect::set(
            &info,
            &JsValue::from_str("is_node"),
            &JsValue::from_bool(Self::is_node()),
        ).unwrap();

        if let Some(ua) = Self::get_user_agent() {
            js_sys::Reflect::set(
                &info,
                &JsValue::from_str("user_agent"),
                &JsValue::from_str(&ua),
            ).unwrap();
        }

        js_sys::Reflect::set(
            &info,
            &JsValue::from_str("has_local_storage"),
            &JsValue::from_bool(Self::has_local_storage()),
        ).unwrap();

        js_sys::Reflect::set(
            &info,
            &JsValue::from_str("has_web_workers"),
            &JsValue::from_bool(Self::has_web_workers()),
        ).unwrap();

        js_sys::Reflect::set(
            &info,
            &JsValue::from_str("has_indexed_db"),
            &JsValue::from_bool(Self::has_indexed_db()),
        ).unwrap();

        info
    }
}

/// Cross-browser compatibility tester
pub struct CompatibilityTester {
    required_features: Vec<String>,
    optional_features: Vec<String>,
}

impl CompatibilityTester {
    /// Create a new compatibility tester
    pub fn new() -> Self {
        Self {
            required_features: vec![
                "wasm".to_string(),
                "javascript".to_string(),
            ],
            optional_features: vec![
                "localStorage".to_string(),
                "webWorkers".to_string(),
                "indexedDB".to_string(),
            ],
        }
    }

    /// Add a required feature
    pub fn require_feature(mut self, feature: &str) -> Self {
        self.required_features.push(feature.to_string());
        self
    }

    /// Add an optional feature
    pub fn optional_feature(mut self, feature: &str) -> Self {
        self.optional_features.push(feature.to_string());
        self
    }

    /// Check compatibility
    pub fn check(&self) -> CompatibilityResult {
        let mut missing_required = Vec::new();
        let mut missing_optional = Vec::new();

        // Check required features
        for feature in &self.required_features {
            if !self.is_feature_available(feature) {
                missing_required.push(feature.clone());
            }
        }

        // Check optional features
        for feature in &self.optional_features {
            if !self.is_feature_available(feature) {
                missing_optional.push(feature.clone());
            }
        }

        CompatibilityResult {
            is_compatible: missing_required.is_empty(),
            missing_required,
            missing_optional,
        }
    }

    fn is_feature_available(&self, feature: &str) -> bool {
        match feature {
            "wasm" => true, // If we're running, WASM is available
            "javascript" => true, // If we're running, JS is available
            "localStorage" => BrowserEnvironment::has_local_storage(),
            "webWorkers" => BrowserEnvironment::has_web_workers(),
            "indexedDB" => BrowserEnvironment::has_indexed_db(),
            _ => false,
        }
    }
}

/// Result of compatibility check
pub struct CompatibilityResult {
    pub is_compatible: bool,
    pub missing_required: Vec<String>,
    pub missing_optional: Vec<String>,
}

impl CompatibilityResult {
    /// Convert to JavaScript object
    pub fn to_js_object(&self) -> js_sys::Object {
        let obj = js_sys::Object::new();

        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("is_compatible"),
            &JsValue::from_bool(self.is_compatible),
        ).unwrap();

        let required_arr = js_sys::Array::new();
        for feature in &self.missing_required {
            required_arr.push(&JsValue::from_str(feature));
        }
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("missing_required"),
            &required_arr,
        ).unwrap();

        let optional_arr = js_sys::Array::new();
        for feature in &self.missing_optional {
            optional_arr.push(&JsValue::from_str(feature));
        }
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("missing_optional"),
            &optional_arr,
        ).unwrap();

        obj
    }
}

/// Integration test suite runner
#[wasm_bindgen]
pub struct TestSuiteRunner {
    suites: Vec<TestSuite>,
}

struct TestSuite {
    name: String,
    tests: Vec<String>,
}

#[wasm_bindgen]
impl TestSuiteRunner {
    /// Create a new test suite runner
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            suites: Vec::new(),
        }
    }

    /// Register a test suite
    pub fn register_suite(&mut self, name: &str, tests: js_sys::Array) {
        let mut test_names = Vec::new();
        for i in 0..tests.length() {
            if let Some(test_name) = tests.get(i).as_string() {
                test_names.push(test_name);
            }
        }

        self.suites.push(TestSuite {
            name: name.to_string(),
            tests: test_names,
        });
    }

    /// Get registered suites
    pub fn get_suites(&self) -> js_sys::Array {
        let suites = js_sys::Array::new();
        
        for suite in &self.suites {
            let obj = js_sys::Object::new();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("name"),
                &JsValue::from_str(&suite.name),
            ).unwrap();

            let tests_arr = js_sys::Array::new();
            for test in &suite.tests {
                tests_arr.push(&JsValue::from_str(test));
            }
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("tests"),
                &tests_arr,
            ).unwrap();

            suites.push(&obj);
        }

        suites
    }

    /// Get total test count
    pub fn get_test_count(&self) -> usize {
        self.suites.iter().map(|s| s.tests.len()).sum()
    }
}
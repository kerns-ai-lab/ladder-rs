//! Browser compatibility utilities and polyfills
//! 
//! This module provides utilities for ensuring consistent behavior across
//! different browser environments, including feature detection and polyfills.

use wasm_bindgen::prelude::*;
use web_sys::{window, Window, Storage, Performance, console};
use js_sys::{Function, Object, Reflect};

/// Browser compatibility utilities
#[wasm_bindgen]
pub struct CrossBrowserCompat;

#[wasm_bindgen]
impl CrossBrowserCompat {
    /// Initialize browser compatibility layer
    pub fn init() {
        Self::setup_console_polyfills();
        Self::setup_performance_polyfills();
    }
    
    /// Get browser information
    pub fn get_browser_info() -> Option<String> {
        if let Some(window) = window() {
            if let Ok(user_agent) = window.navigator().user_agent() {
                let user_agent = user_agent.to_lowercase();
                
                let browser = if user_agent.contains("firefox") {
                    "Firefox"
                } else if user_agent.contains("edg/") {
                    "Edge"
                } else if user_agent.contains("chrome") && !user_agent.contains("edg/") {
                    "Chrome"
                } else if user_agent.contains("safari") && !user_agent.contains("chrome") {
                    "Safari"
                } else if user_agent.contains("opera") || user_agent.contains("opr/") {
                    "Opera"
                } else {
                    "Unknown"
                };
                
                return Some(browser.to_string());
            }
        }
        None
    }
    
    /// Check if a browser feature is available
    pub fn has_feature(feature: &str) -> bool {
        if let Some(window) = window() {
            match feature {
                "localStorage" => Self::check_storage(&window, "localStorage"),
                "sessionStorage" => Self::check_storage(&window, "sessionStorage"),
                "indexedDB" => Reflect::has(&window, &"indexedDB".into()).unwrap_or(false),
                "webWorker" => Reflect::has(&window, &"Worker".into()).unwrap_or(false),
                "serviceWorker" => {
                    let navigator = window.navigator();
                    Reflect::has(&navigator, &"serviceWorker".into()).unwrap_or(false)
                }
                "webAssembly" => Reflect::has(&window, &"WebAssembly".into()).unwrap_or(false),
                "performance" => window.performance().is_some(),
                "promise" => Reflect::has(&window, &"Promise".into()).unwrap_or(false),
                "fetch" => Reflect::has(&window, &"fetch".into()).unwrap_or(false),
                _ => false,
            }
        } else {
            false
        }
    }
    
    /// Get safe storage (localStorage with fallback to sessionStorage)
    pub fn get_safe_storage() -> Option<Storage> {
        if let Some(window) = window() {
            // Try localStorage first
            if let Ok(Some(storage)) = window.local_storage() {
                if Self::test_storage(&storage) {
                    return Some(storage);
                }
            }
            
            // Fallback to sessionStorage
            if let Ok(Some(storage)) = window.session_storage() {
                if Self::test_storage(&storage) {
                    return Some(storage);
                }
            }
        }
        None
    }
    
    /// Get performance object with polyfill
    pub fn get_performance() -> Option<Performance> {
        window()?.performance()
    }
    
    /// Safe console log
    pub fn console_log(message: &str) {
        console::log_1(&message.into());
    }
    
    /// Safe console error
    pub fn console_error(message: &str) {
        console::error_1(&message.into());
    }
    
    /// Safe console warn
    pub fn console_warn(message: &str) {
        console::warn_1(&message.into());
    }
}

// Private implementation methods
impl CrossBrowserCompat {
    fn setup_console_polyfills() {
        if let Some(window) = window() {
            let console_obj = Reflect::get(&window, &"console".into()).unwrap_or(JsValue::UNDEFINED);
            
            if console_obj.is_undefined() {
                // Create console object if it doesn't exist
                let console = Object::new();
                let noop = Function::new_no_args("");
                
                let _ = Reflect::set(&console, &"log".into(), &noop);
                let _ = Reflect::set(&console, &"error".into(), &noop);
                let _ = Reflect::set(&console, &"warn".into(), &noop);
                let _ = Reflect::set(&console, &"info".into(), &noop);
                let _ = Reflect::set(&console, &"debug".into(), &noop);
                
                let _ = Reflect::set(&window, &"console".into(), &console);
            }
        }
    }
    
    fn setup_performance_polyfills() {
        if let Some(window) = window() {
            if window.performance().is_none() {
                // Create a basic performance polyfill
                let perf = Object::new();
                
                // Create performance.now() polyfill using Date.now()
                let now_fn = Function::new_no_args("return Date.now()");
                let _ = Reflect::set(&perf, &"now".into(), &now_fn);
                
                let _ = Reflect::set(&window, &"performance".into(), &perf);
            }
        }
    }
    
    fn check_storage(window: &Window, storage_type: &str) -> bool {
        match storage_type {
            "localStorage" => {
                if let Ok(Some(storage)) = window.local_storage() {
                    Self::test_storage(&storage)
                } else {
                    false
                }
            }
            "sessionStorage" => {
                if let Ok(Some(storage)) = window.session_storage() {
                    Self::test_storage(&storage)
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    
    fn test_storage(storage: &Storage) -> bool {
        let test_key = "__ladder_rs_test__";
        match storage.set_item(test_key, "test") {
            Ok(_) => {
                let _ = storage.remove_item(test_key);
                true
            }
            Err(_) => false,
        }
    }
}

/// Browser-specific event handling utilities
#[wasm_bindgen]
pub struct EventCompat;

#[wasm_bindgen]
impl EventCompat {
    /// Add event listener with compatibility handling
    pub fn add_listener(target: &JsValue, event_type: &str, handler: &Function) -> Result<(), JsValue> {
        if let Some(element) = target.dyn_ref::<web_sys::EventTarget>() {
            element.add_event_listener_with_callback(event_type, handler)?;
            Ok(())
        } else {
            Err(JsValue::from_str("Invalid event target"))
        }
    }
    
    /// Remove event listener with compatibility handling
    pub fn remove_listener(target: &JsValue, event_type: &str, handler: &Function) -> Result<(), JsValue> {
        if let Some(element) = target.dyn_ref::<web_sys::EventTarget>() {
            element.remove_event_listener_with_callback(event_type, handler)?;
            Ok(())
        } else {
            Err(JsValue::from_str("Invalid event target"))
        }
    }
    
    /// Create custom event with compatibility
    pub fn create_custom_event(event_type: &str, detail: &JsValue) -> Result<web_sys::CustomEvent, JsValue> {
        let event_init = web_sys::CustomEventInit::new();
        event_init.set_detail(detail);
        
        let event = web_sys::CustomEvent::new_with_event_init_dict(event_type, &event_init)?;
        Ok(event)
    }
}

/// Performance monitoring with browser compatibility
#[wasm_bindgen]
pub struct PerformanceCompat;

#[wasm_bindgen]
impl PerformanceCompat {
    /// Get current timestamp with fallback
    pub fn now() -> f64 {
        if let Some(perf) = CrossBrowserCompat::get_performance() {
            perf.now()
        } else {
            js_sys::Date::now()
        }
    }
    
    /// Mark performance timing
    pub fn mark(name: &str) {
        if let Some(perf) = CrossBrowserCompat::get_performance() {
            let _ = perf.mark(name);
        }
    }
    
    /// Measure performance between marks
    pub fn measure(name: &str, start_mark: &str, end_mark: &str) {
        if let Some(perf) = CrossBrowserCompat::get_performance() {
            let _ = perf.measure_with_start_mark_and_end_mark(name, start_mark, end_mark);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    
    wasm_bindgen_test_configure!(run_in_browser);
    
    #[wasm_bindgen_test]
    fn test_browser_detection() {
        let browser = CrossBrowserCompat::get_browser_info();
        assert!(browser.is_some());
    }
    
    #[wasm_bindgen_test]
    fn test_feature_detection() {
        // These should be available in test environment
        assert!(CrossBrowserCompat::has_feature("promise"));
        assert!(CrossBrowserCompat::has_feature("webAssembly"));
    }
    
    #[wasm_bindgen_test]
    fn test_console_methods() {
        // Should not panic
        CrossBrowserCompat::console_log("Test log");
        CrossBrowserCompat::console_error("Test error");
        CrossBrowserCompat::console_warn("Test warning");
    }
    
    #[wasm_bindgen_test]
    fn test_performance_now() {
        let time1 = PerformanceCompat::now();
        let time2 = PerformanceCompat::now();
        assert!(time2 >= time1);
    }
}
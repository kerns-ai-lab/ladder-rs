//! Performance monitoring interface for JavaScript

use wasm_bindgen::prelude::*;
use js_sys::*;
use std::collections::HashMap;

/// Performance monitor interface
#[wasm_bindgen(js_name = "PerformanceMonitor")]
pub struct JsPerformanceMonitorInterface {
    timers: HashMap<String, f64>,
    stats: HashMap<String, f64>,
}

#[wasm_bindgen(js_class = "PerformanceMonitor")]
impl JsPerformanceMonitorInterface {
    /// Creates a new performance monitor
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            timers: HashMap::new(),
            stats: HashMap::new(),
        }
    }
    
    /// Start timing an operation
    #[wasm_bindgen(js_name = "startTimer")]
    pub fn start_timer(&mut self, name: &str) {
        let now = Date::now();
        self.timers.insert(name.to_string(), now);
    }
    
    /// End timing and return duration
    #[wasm_bindgen(js_name = "endTimer")]
    pub fn end_timer(&mut self, name: &str) -> f64 {
        let now = Date::now();
        if let Some(start_time) = self.timers.remove(name) {
            let duration = now - start_time;
            self.stats.insert(name.to_string(), duration);
            duration
        } else {
            0.0
        }
    }
    
    /// Get performance statistics
    #[wasm_bindgen(js_name = "getStatistics")]
    pub fn get_statistics(&self) -> Object {
        let obj = Object::new();
        for (key, value) in &self.stats {
            Reflect::set(&obj, &JsValue::from_str(key), &JsValue::from_f64(*value)).unwrap();
        }
        obj
    }
    
    /// Get memory usage (simplified)
    #[wasm_bindgen(js_name = "getMemoryUsage")]
    pub fn get_memory_usage(&self) -> u32 {
        1024  // Placeholder - would use actual memory APIs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_monitor() {
        let mut monitor = JsPerformanceMonitorInterface::new();
        monitor.start_timer("test");
        let duration = monitor.end_timer("test");
        assert!(duration >= 0.0);
    }
}
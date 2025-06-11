//! Performance measurement utilities for WASM tests

use js_sys::{Array, Date, Object, Reflect};
use wasm_bindgen::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

/// Performance timer for benchmarking
#[wasm_bindgen]
pub struct PerformanceTimer {
    start_time: f64,
    laps: Vec<(String, f64)>,
    markers: HashMap<String, f64>,
}

#[wasm_bindgen]
impl PerformanceTimer {
    /// Create a new performance timer
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            start_time: Date::now(),
            laps: Vec::new(),
            markers: HashMap::new(),
        }
    }

    /// Record a lap time with label
    pub fn lap(&mut self, label: &str) -> f64 {
        let current_time = Date::now();
        let elapsed = current_time - self.start_time;
        self.laps.push((label.to_string(), elapsed));
        elapsed
    }

    /// Mark a specific point in time
    pub fn mark(&mut self, label: &str) {
        self.markers.insert(label.to_string(), Date::now());
    }

    /// Get time between two markers
    pub fn time_between(&self, start_marker: &str, end_marker: &str) -> Option<f64> {
        let start = self.markers.get(start_marker)?;
        let end = self.markers.get(end_marker)?;
        Some(end - start)
    }

    /// Get total elapsed time
    pub fn elapsed(&self) -> f64 {
        Date::now() - self.start_time
    }

    /// Get all lap times as JavaScript object
    pub fn get_laps(&self) -> Result<Object, JsValue> {
        let obj = Object::new();
        for (label, time) in &self.laps {
            Reflect::set(&obj, &JsValue::from_str(label), &JsValue::from_f64(*time))?;
        }
        Ok(obj)
    }

    /// Get lap times as array
    pub fn get_laps_array(&self) -> Array {
        let arr = Array::new();
        for (label, time) in &self.laps {
            let lap_obj = Object::new();
            Reflect::set(&lap_obj, &JsValue::from_str("label"), &JsValue::from_str(label)).unwrap();
            Reflect::set(&lap_obj, &JsValue::from_str("time"), &JsValue::from_f64(*time)).unwrap();
            arr.push(&lap_obj);
        }
        arr
    }

    /// Get performance summary
    pub fn get_summary(&self) -> Result<Object, JsValue> {
        let summary = Object::new();
        
        let total_time = self.elapsed();
        Reflect::set(&summary, &JsValue::from_str("total_ms"), &JsValue::from_f64(total_time))?;
        
        if !self.laps.is_empty() {
            let avg_lap = total_time / self.laps.len() as f64;
            Reflect::set(&summary, &JsValue::from_str("average_lap_ms"), &JsValue::from_f64(avg_lap))?;
            Reflect::set(&summary, &JsValue::from_str("lap_count"), &JsValue::from_f64(self.laps.len() as f64))?;
            
            // Find min and max lap times
            let mut min_lap = f64::MAX;
            let mut max_lap = f64::MIN;
            let mut min_label = "";
            let mut max_label = "";
            
            for (i, (label, _)) in self.laps.iter().enumerate() {
                let lap_time = if i == 0 {
                    self.laps[0].1
                } else {
                    self.laps[i].1 - self.laps[i - 1].1
                };
                
                if lap_time < min_lap {
                    min_lap = lap_time;
                    min_label = label;
                }
                if lap_time > max_lap {
                    max_lap = lap_time;
                    max_label = label;
                }
            }
            
            let min_obj = Object::new();
            Reflect::set(&min_obj, &JsValue::from_str("label"), &JsValue::from_str(min_label))?;
            Reflect::set(&min_obj, &JsValue::from_str("time"), &JsValue::from_f64(min_lap))?;
            Reflect::set(&summary, &JsValue::from_str("min_lap"), &min_obj)?;
            
            let max_obj = Object::new();
            Reflect::set(&max_obj, &JsValue::from_str("label"), &JsValue::from_str(max_label))?;
            Reflect::set(&max_obj, &JsValue::from_str("time"), &JsValue::from_f64(max_lap))?;
            Reflect::set(&summary, &JsValue::from_str("max_lap"), &max_obj)?;
        }
        
        Ok(summary)
    }

    /// Reset the timer
    pub fn reset(&mut self) {
        self.start_time = Date::now();
        self.laps.clear();
        self.markers.clear();
    }
}

/// Result of a performance measurement
pub struct PerformanceResult<T> {
    pub value: T,
    pub duration_ms: f64,
}

/// Measure the performance of a function
pub fn measure_performance<T, F: FnOnce() -> T>(f: F) -> PerformanceResult<T> {
    let start = Date::now();
    let value = f();
    let end = Date::now();
    
    PerformanceResult {
        value,
        duration_ms: end - start,
    }
}

/// Benchmark runner for comparative performance testing
#[wasm_bindgen]
pub struct BenchmarkRunner {
    results: RefCell<HashMap<String, Vec<f64>>>,
    iterations: u32,
}

#[wasm_bindgen]
impl BenchmarkRunner {
    /// Create a new benchmark runner
    #[wasm_bindgen(constructor)]
    pub fn new(iterations: u32) -> Self {
        Self {
            results: RefCell::new(HashMap::new()),
            iterations,
        }
    }

    /// Run a benchmark
    pub fn run_benchmark(&self, name: &str, test_fn: &js_sys::Function) -> Result<Object, JsValue> {
        let mut times = Vec::new();
        
        for _ in 0..self.iterations {
            let start = Date::now();
            test_fn.call0(&JsValue::NULL)?;
            let duration = Date::now() - start;
            times.push(duration);
        }
        
        self.results.borrow_mut().insert(name.to_string(), times.clone());
        
        // Calculate statistics
        let stats = self.calculate_stats(&times);
        Ok(stats)
    }

    /// Compare all benchmarks
    pub fn compare(&self) -> Result<Array, JsValue> {
        let results = self.results.borrow();
        let comparison = Array::new();
        
        for (name, times) in results.iter() {
            let stats = self.calculate_stats(times);
            Reflect::set(&stats, &JsValue::from_str("name"), &JsValue::from_str(name))?;
            comparison.push(&stats);
        }
        
        Ok(comparison)
    }

    /// Get raw results
    pub fn get_raw_results(&self) -> Object {
        let results = self.results.borrow();
        let obj = Object::new();
        
        for (name, times) in results.iter() {
            let times_array = Array::new();
            for time in times {
                times_array.push(&JsValue::from_f64(*time));
            }
            Reflect::set(&obj, &JsValue::from_str(name), &times_array).unwrap();
        }
        
        obj
    }

    /// Clear results
    pub fn clear(&self) {
        self.results.borrow_mut().clear();
    }

    fn calculate_stats(&self, times: &[f64]) -> Object {
        let stats = Object::new();
        
        if times.is_empty() {
            return stats;
        }
        
        let sum: f64 = times.iter().sum();
        let mean = sum / times.len() as f64;
        
        let mut sorted_times = times.to_vec();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let median = if sorted_times.len() % 2 == 0 {
            let mid = sorted_times.len() / 2;
            (sorted_times[mid - 1] + sorted_times[mid]) / 2.0
        } else {
            sorted_times[sorted_times.len() / 2]
        };
        
        let min = sorted_times[0];
        let max = sorted_times[sorted_times.len() - 1];
        
        // Calculate standard deviation
        let variance: f64 = times.iter()
            .map(|t| (t - mean).powi(2))
            .sum::<f64>() / times.len() as f64;
        let std_dev = variance.sqrt();
        
        // Calculate percentiles
        let p95_idx = ((sorted_times.len() as f64 * 0.95) as usize).min(sorted_times.len() - 1);
        let p99_idx = ((sorted_times.len() as f64 * 0.99) as usize).min(sorted_times.len() - 1);
        
        Reflect::set(&stats, &JsValue::from_str("mean"), &JsValue::from_f64(mean)).unwrap();
        Reflect::set(&stats, &JsValue::from_str("median"), &JsValue::from_f64(median)).unwrap();
        Reflect::set(&stats, &JsValue::from_str("min"), &JsValue::from_f64(min)).unwrap();
        Reflect::set(&stats, &JsValue::from_str("max"), &JsValue::from_f64(max)).unwrap();
        Reflect::set(&stats, &JsValue::from_str("std_dev"), &JsValue::from_f64(std_dev)).unwrap();
        Reflect::set(&stats, &JsValue::from_str("p95"), &JsValue::from_f64(sorted_times[p95_idx])).unwrap();
        Reflect::set(&stats, &JsValue::from_str("p99"), &JsValue::from_f64(sorted_times[p99_idx])).unwrap();
        Reflect::set(&stats, &JsValue::from_str("iterations"), &JsValue::from_f64(times.len() as f64)).unwrap();
        
        stats
    }
}

/// Memory usage tracker
#[wasm_bindgen]
pub struct MemoryTracker {
    initial_memory: Option<usize>,
    snapshots: Vec<(String, usize)>,
}

#[wasm_bindgen]
impl MemoryTracker {
    /// Create a new memory tracker
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let initial = Self::get_current_memory();
        Self {
            initial_memory: initial,
            snapshots: Vec::new(),
        }
    }

    /// Take a memory snapshot
    pub fn snapshot(&mut self, label: &str) {
        if let Some(memory) = Self::get_current_memory() {
            self.snapshots.push((label.to_string(), memory));
        }
    }

    /// Get memory usage report
    pub fn get_report(&self) -> Object {
        let report = Object::new();
        
        if let Some(initial) = self.initial_memory {
            Reflect::set(&report, &JsValue::from_str("initial_bytes"), &JsValue::from_f64(initial as f64)).unwrap();
            
            if let Some(current) = Self::get_current_memory() {
                Reflect::set(&report, &JsValue::from_str("current_bytes"), &JsValue::from_f64(current as f64)).unwrap();
                let diff = (current as i64 - initial as i64) as f64;
                Reflect::set(&report, &JsValue::from_str("difference_bytes"), &JsValue::from_f64(diff)).unwrap();
            }
        }
        
        let snapshots_arr = Array::new();
        for (label, memory) in &self.snapshots {
            let snapshot_obj = Object::new();
            Reflect::set(&snapshot_obj, &JsValue::from_str("label"), &JsValue::from_str(label)).unwrap();
            Reflect::set(&snapshot_obj, &JsValue::from_str("bytes"), &JsValue::from_f64(*memory as f64)).unwrap();
            snapshots_arr.push(&snapshot_obj);
        }
        Reflect::set(&report, &JsValue::from_str("snapshots"), &snapshots_arr).unwrap();
        
        report
    }

    fn get_current_memory() -> Option<usize> {
        // In WASM, we can try to get memory from the memory object
        if let Ok(memory) = js_sys::Reflect::get(&wasm_bindgen::memory(), &JsValue::from_str("buffer")) {
            if let Ok(buffer) = memory.dyn_into::<js_sys::ArrayBuffer>() {
                return Some(buffer.byte_length() as usize);
            }
        }
        None
    }
}
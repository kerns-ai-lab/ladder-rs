//! Coverage tracking utilities for test code coverage analysis

use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

/// Coverage tracker for function-level coverage
#[wasm_bindgen]
pub struct CoverageTracker {
    local_functions: RefCell<HashSet<String>>,
    tracked_functions: RefCell<HashMap<String, FunctionCoverage>>,
}

#[derive(Clone)]
struct FunctionCoverage {
    name: String,
    calls: usize,
    paths_taken: HashSet<String>,
    total_paths: Option<usize>,
}

#[wasm_bindgen]
impl CoverageTracker {
    /// Create a new coverage tracker
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            local_functions: RefCell::new(HashSet::new()),
            tracked_functions: RefCell::new(HashMap::new()),
        }
    }

    /// Register a function for tracking
    pub fn register_function(&self, name: &str, total_paths: Option<u32>) {
        self.local_functions.borrow_mut().insert(name.to_string());
        self.tracked_functions.borrow_mut().insert(
            name.to_string(),
            FunctionCoverage {
                name: name.to_string(),
                calls: 0,
                paths_taken: HashSet::new(),
                total_paths: total_paths.map(|p| p as usize),
            },
        );
    }

    /// Track a function call
    pub fn track_call(&self, function_name: &str) {
        if let Some(coverage) = self.tracked_functions.borrow_mut().get_mut(function_name) {
            coverage.calls += 1;
        } else {
            // Auto-register if not already tracked
            self.tracked_functions.borrow_mut().insert(
                function_name.to_string(),
                FunctionCoverage {
                    name: function_name.to_string(),
                    calls: 1,
                    paths_taken: HashSet::new(),
                    total_paths: None,
                },
            );
        }
    }

    /// Track a specific path within a function
    pub fn track_path(&self, function_name: &str, path_id: &str) {
        if let Some(coverage) = self.tracked_functions.borrow_mut().get_mut(function_name) {
            coverage.paths_taken.insert(path_id.to_string());
        }
    }

    /// Get coverage percentage
    pub fn get_coverage_percentage(&self) -> f64 {
        let functions = self.tracked_functions.borrow();
        if functions.is_empty() {
            return 0.0;
        }

        let covered = functions.values().filter(|f| f.calls > 0).count();
        (covered as f64 / functions.len() as f64) * 100.0
    }

    /// Get detailed coverage report
    pub fn get_report(&self) -> js_sys::Object {
        let report = js_sys::Object::new();
        let functions = self.tracked_functions.borrow();

        // Overall statistics
        let total_functions = functions.len() as f64;
        let covered_functions = functions.values().filter(|f| f.calls > 0).count() as f64;
        let coverage_percentage = if total_functions > 0.0 {
            (covered_functions / total_functions) * 100.0
        } else {
            0.0
        };

        js_sys::Reflect::set(
            &report,
            &JsValue::from_str("total_functions"),
            &JsValue::from_f64(total_functions),
        ).unwrap();
        js_sys::Reflect::set(
            &report,
            &JsValue::from_str("covered_functions"),
            &JsValue::from_f64(covered_functions),
        ).unwrap();
        js_sys::Reflect::set(
            &report,
            &JsValue::from_str("coverage_percentage"),
            &JsValue::from_f64(coverage_percentage),
        ).unwrap();

        // Per-function details
        let details = js_sys::Array::new();
        for (name, coverage) in functions.iter() {
            let func_obj = js_sys::Object::new();
            js_sys::Reflect::set(
                &func_obj,
                &JsValue::from_str("name"),
                &JsValue::from_str(name),
            ).unwrap();
            js_sys::Reflect::set(
                &func_obj,
                &JsValue::from_str("calls"),
                &JsValue::from_f64(coverage.calls as f64),
            ).unwrap();
            js_sys::Reflect::set(
                &func_obj,
                &JsValue::from_str("covered"),
                &JsValue::from_bool(coverage.calls > 0),
            ).unwrap();

            // Path coverage if available
            if let Some(total_paths) = coverage.total_paths {
                let path_coverage = if total_paths > 0 {
                    (coverage.paths_taken.len() as f64 / total_paths as f64) * 100.0
                } else {
                    100.0
                };
                js_sys::Reflect::set(
                    &func_obj,
                    &JsValue::from_str("path_coverage"),
                    &JsValue::from_f64(path_coverage),
                ).unwrap();
                js_sys::Reflect::set(
                    &func_obj,
                    &JsValue::from_str("paths_taken"),
                    &JsValue::from_f64(coverage.paths_taken.len() as f64),
                ).unwrap();
                js_sys::Reflect::set(
                    &func_obj,
                    &JsValue::from_str("total_paths"),
                    &JsValue::from_f64(total_paths as f64),
                ).unwrap();
            }

            details.push(&func_obj);
        }
        js_sys::Reflect::set(&report, &JsValue::from_str("functions"), &details).unwrap();

        report
    }

    /// Reset coverage data
    pub fn reset(&self) {
        self.tracked_functions.borrow_mut().clear();
    }

    /// Export coverage data as JSON
    pub fn export_json(&self) -> String {
        let report = self.get_report();
        js_sys::JSON::stringify(&report)
            .map(|s| s.as_string().unwrap_or_default())
            .unwrap_or_default()
    }
}

/// Branch coverage tracker
#[wasm_bindgen]
pub struct BranchCoverageTracker {
    branches: RefCell<HashMap<String, BranchInfo>>,
}

struct BranchInfo {
    location: String,
    true_count: usize,
    false_count: usize,
}

#[wasm_bindgen]
impl BranchCoverageTracker {
    /// Create a new branch coverage tracker
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            branches: RefCell::new(HashMap::new()),
        }
    }

    /// Track a branch execution
    pub fn track_branch(&self, location: &str, condition: bool) {
        let mut branches = self.branches.borrow_mut();
        let branch = branches.entry(location.to_string()).or_insert(BranchInfo {
            location: location.to_string(),
            true_count: 0,
            false_count: 0,
        });

        if condition {
            branch.true_count += 1;
        } else {
            branch.false_count += 1;
        }
    }

    /// Get branch coverage statistics
    pub fn get_statistics(&self) -> js_sys::Object {
        let branches = self.branches.borrow();
        let stats = js_sys::Object::new();

        let total_branches = branches.len() as f64;
        let mut covered_branches = 0;
        let mut partial_branches = 0;

        for branch in branches.values() {
            if branch.true_count > 0 && branch.false_count > 0 {
                covered_branches += 1;
            } else if branch.true_count > 0 || branch.false_count > 0 {
                partial_branches += 1;
            }
        }

        let coverage_percentage = if total_branches > 0.0 {
            (covered_branches as f64 / total_branches) * 100.0
        } else {
            0.0
        };

        js_sys::Reflect::set(
            &stats,
            &JsValue::from_str("total_branches"),
            &JsValue::from_f64(total_branches),
        ).unwrap();
        js_sys::Reflect::set(
            &stats,
            &JsValue::from_str("covered_branches"),
            &JsValue::from_f64(covered_branches as f64),
        ).unwrap();
        js_sys::Reflect::set(
            &stats,
            &JsValue::from_str("partial_branches"),
            &JsValue::from_f64(partial_branches as f64),
        ).unwrap();
        js_sys::Reflect::set(
            &stats,
            &JsValue::from_str("coverage_percentage"),
            &JsValue::from_f64(coverage_percentage),
        ).unwrap();

        stats
    }

    /// Get detailed branch report
    pub fn get_report(&self) -> js_sys::Array {
        let branches = self.branches.borrow();
        let report = js_sys::Array::new();

        for (location, info) in branches.iter() {
            let branch_obj = js_sys::Object::new();
            js_sys::Reflect::set(
                &branch_obj,
                &JsValue::from_str("location"),
                &JsValue::from_str(location),
            ).unwrap();
            js_sys::Reflect::set(
                &branch_obj,
                &JsValue::from_str("true_count"),
                &JsValue::from_f64(info.true_count as f64),
            ).unwrap();
            js_sys::Reflect::set(
                &branch_obj,
                &JsValue::from_str("false_count"),
                &JsValue::from_f64(info.false_count as f64),
            ).unwrap();
            js_sys::Reflect::set(
                &branch_obj,
                &JsValue::from_str("both_covered"),
                &JsValue::from_bool(info.true_count > 0 && info.false_count > 0),
            ).unwrap();

            report.push(&branch_obj);
        }

        report
    }

    /// Reset branch coverage data
    pub fn reset(&self) {
        self.branches.borrow_mut().clear();
    }
}

/// Coverage report generator
#[wasm_bindgen]
pub struct CoverageReport {
    function_coverage: CoverageTracker,
    branch_coverage: BranchCoverageTracker,
    start_time: f64,
}

#[wasm_bindgen]
impl CoverageReport {
    /// Create a new coverage report
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            function_coverage: CoverageTracker::new(),
            branch_coverage: BranchCoverageTracker::new(),
            start_time: js_sys::Date::now(),
        }
    }

    /// Get function coverage tracker
    pub fn functions(&self) -> CoverageTracker {
        CoverageTracker::new() // Return a new instance for WASM compatibility
    }

    /// Get branch coverage tracker
    pub fn branches(&self) -> BranchCoverageTracker {
        BranchCoverageTracker::new() // Return a new instance for WASM compatibility
    }

    /// Generate complete coverage report
    pub fn generate(&self) -> js_sys::Object {
        let report = js_sys::Object::new();

        // Add metadata
        js_sys::Reflect::set(
            &report,
            &JsValue::from_str("generated_at"),
            &JsValue::from_f64(js_sys::Date::now()),
        ).unwrap();
        js_sys::Reflect::set(
            &report,
            &JsValue::from_str("duration_ms"),
            &JsValue::from_f64(js_sys::Date::now() - self.start_time),
        ).unwrap();

        // Add function coverage
        js_sys::Reflect::set(
            &report,
            &JsValue::from_str("function_coverage"),
            &self.function_coverage.get_report(),
        ).unwrap();

        // Add branch coverage
        let branch_stats = self.branch_coverage.get_statistics();
        js_sys::Reflect::set(
            &report,
            &JsValue::from_str("branch_coverage"),
            &branch_stats,
        ).unwrap();

        // Calculate overall coverage
        let func_percentage = self.function_coverage.get_coverage_percentage();
        let branch_percentage = js_sys::Reflect::get(&branch_stats, &JsValue::from_str("coverage_percentage"))
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        
        let overall_percentage = (func_percentage + branch_percentage) / 2.0;
        js_sys::Reflect::set(
            &report,
            &JsValue::from_str("overall_coverage"),
            &JsValue::from_f64(overall_percentage),
        ).unwrap();

        report
    }

    /// Export report as HTML
    pub fn export_html(&self) -> String {
        let report = self.generate();
        let json = js_sys::JSON::stringify(&report)
            .map(|s| s.as_string().unwrap_or_default())
            .unwrap_or_default();

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Coverage Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .summary {{ background: #f0f0f0; padding: 10px; border-radius: 5px; }}
        .covered {{ color: green; }}
        .uncovered {{ color: red; }}
        .partial {{ color: orange; }}
        table {{ border-collapse: collapse; width: 100%; margin-top: 20px; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
    </style>
</head>
<body>
    <h1>Coverage Report</h1>
    <div class="summary">
        <pre>{}</pre>
    </div>
</body>
</html>"#,
            json
        )
    }
}

/// Global coverage tracking
static mut GLOBAL_COVERAGE: Option<CoverageReport> = None;

/// Initialize global coverage tracking
pub fn init_global_coverage() {
    unsafe {
        GLOBAL_COVERAGE = Some(CoverageReport::new());
    }
}

/// Get global coverage report
pub fn get_global_coverage() -> Option<js_sys::Object> {
    unsafe {
        GLOBAL_COVERAGE.as_ref().map(|c| c.generate())
    }
}

/// Track function in global coverage
pub fn track_function(name: &str) {
    unsafe {
        if let Some(ref coverage) = GLOBAL_COVERAGE {
            coverage.function_coverage.track_call(name);
        }
    }
}

/// Track branch in global coverage
pub fn track_branch(location: &str, condition: bool) {
    unsafe {
        if let Some(ref coverage) = GLOBAL_COVERAGE {
            coverage.branch_coverage.track_branch(location, condition);
        }
    }
}
//! Performance tracking and reporting module
//!
//! This module provides utilities for tracking performance metrics,
//! generating reports, and detecting performance regressions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Get the current timestamp in milliseconds, using js_sys::Date in browser WASM
/// to avoid the panic from SystemTime::now() which requires WASI clock.
#[cfg(target_arch = "wasm32")]
fn current_timestamp_ms() -> f64 {
    js_sys::Date::now()
}

#[cfg(not(target_arch = "wasm32"))]
fn current_timestamp_ms() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as f64
}

/// Performance metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub operation: String,
    pub timestamp: u64,
    pub duration_ms: f64,
    pub iterations: u32,
    pub ops_per_second: f64,
    pub memory_used_bytes: Option<u64>,
    pub metadata: HashMap<String, String>,
}

impl PerformanceMetric {
    pub fn new(operation: &str, duration_ms: f64, iterations: u32) -> Self {
        let ops_per_second = (iterations as f64 * 1000.0) / duration_ms;
        let timestamp = (current_timestamp_ms() / 1000.0) as u64;

        Self {
            operation: operation.to_string(),
            timestamp,
            duration_ms,
            iterations,
            ops_per_second,
            memory_used_bytes: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_memory(mut self, bytes: u64) -> Self {
        self.memory_used_bytes = Some(bytes);
        self
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Performance baseline for regression detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    pub operation: String,
    pub min_ops_per_second: f64,
    pub max_duration_ms: f64,
    pub max_memory_bytes: Option<u64>,
}

/// Performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub timestamp: u64,
    pub environment: String,
    pub metrics: Vec<PerformanceMetric>,
    pub regressions: Vec<RegressionResult>,
    pub summary: ReportSummary,
}

/// Regression detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionResult {
    pub operation: String,
    pub baseline: PerformanceBaseline,
    pub actual: PerformanceMetric,
    pub regression_type: RegressionType,
    pub severity: RegressionSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionType {
    Speed,
    Memory,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionSeverity {
    Minor,    // < 10% regression
    Major,    // 10-50% regression
    Critical, // > 50% regression
}

/// Summary statistics for the report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_operations: usize,
    pub passed: usize,
    pub regressions: usize,
    pub critical_regressions: usize,
    pub average_performance_ratio: f64,
}

/// Performance tracker for collecting and analyzing metrics
#[wasm_bindgen]
pub struct PerformanceTracker {
    metrics: Vec<PerformanceMetric>,
    baselines: HashMap<String, PerformanceBaseline>,
    environment: String,
}

#[wasm_bindgen]
impl PerformanceTracker {
    #[wasm_bindgen(constructor)]
    pub fn new(environment: String) -> Self {
        Self {
            metrics: Vec::new(),
            baselines: Self::default_baselines(),
            environment,
        }
    }

    /// Record a performance metric
    pub fn record_metric(&mut self, operation: &str, duration_ms: f64, iterations: u32) {
        let metric = PerformanceMetric::new(operation, duration_ms, iterations);
        self.metrics.push(metric);
    }

    /// Record a performance metric with memory usage
    pub fn record_metric_with_memory(
        &mut self,
        operation: &str,
        duration_ms: f64,
        iterations: u32,
        memory_bytes: u64,
    ) {
        let metric =
            PerformanceMetric::new(operation, duration_ms, iterations).with_memory(memory_bytes);
        self.metrics.push(metric);
    }

    /// Add or update a performance baseline
    pub fn set_baseline(&mut self, operation: &str, min_ops_per_second: f64, max_duration_ms: f64) {
        self.baselines.insert(
            operation.to_string(),
            PerformanceBaseline {
                operation: operation.to_string(),
                min_ops_per_second,
                max_duration_ms,
                max_memory_bytes: None,
            },
        );
    }

    /// Generate a performance report
    pub fn generate_report(&self) -> String {
        let report = self.create_report();
        serde_json::to_string_pretty(&report).unwrap()
    }

    /// Check for performance regressions
    pub fn check_regressions(&self) -> bool {
        let regressions = self.detect_regressions();
        regressions.is_empty()
    }

    /// Get regression count
    pub fn regression_count(&self) -> usize {
        self.detect_regressions().len()
    }

    /// Export metrics as JSON
    pub fn export_metrics(&self) -> String {
        serde_json::to_string(&self.metrics).unwrap()
    }

    /// Import baselines from JSON
    pub fn import_baselines(&mut self, json: &str) -> Result<(), JsValue> {
        let baselines: HashMap<String, PerformanceBaseline> = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse baselines: {}", e)))?;

        self.baselines = baselines;
        Ok(())
    }
}

// Private implementation methods
impl PerformanceTracker {
    fn default_baselines() -> HashMap<String, PerformanceBaseline> {
        let mut baselines = HashMap::new();

        // System creation baselines
        baselines.insert(
            "create_elo_system".to_string(),
            PerformanceBaseline {
                operation: "create_elo_system".to_string(),
                min_ops_per_second: 10000.0,
                max_duration_ms: 0.1,
                max_memory_bytes: None,
            },
        );

        baselines.insert(
            "create_trueskill_system".to_string(),
            PerformanceBaseline {
                operation: "create_trueskill_system".to_string(),
                min_ops_per_second: 5000.0,
                max_duration_ms: 0.2,
                max_memory_bytes: None,
            },
        );

        // Rating update baselines
        baselines.insert(
            "elo_1v1_update".to_string(),
            PerformanceBaseline {
                operation: "elo_1v1_update".to_string(),
                min_ops_per_second: 1000.0,
                max_duration_ms: 1.0,
                max_memory_bytes: None,
            },
        );

        baselines.insert(
            "trueskill_1v1_update".to_string(),
            PerformanceBaseline {
                operation: "trueskill_1v1_update".to_string(),
                min_ops_per_second: 100.0,
                max_duration_ms: 10.0,
                max_memory_bytes: None,
            },
        );

        // Serialization baselines
        baselines.insert(
            "serialize_100_players".to_string(),
            PerformanceBaseline {
                operation: "serialize_100_players".to_string(),
                min_ops_per_second: 100.0,
                max_duration_ms: 10.0,
                max_memory_bytes: None,
            },
        );

        baselines.insert(
            "deserialize_100_players".to_string(),
            PerformanceBaseline {
                operation: "deserialize_100_players".to_string(),
                min_ops_per_second: 100.0,
                max_duration_ms: 10.0,
                max_memory_bytes: None,
            },
        );

        // Batch operation baselines
        baselines.insert(
            "batch_100_matches".to_string(),
            PerformanceBaseline {
                operation: "batch_100_matches".to_string(),
                min_ops_per_second: 10.0,
                max_duration_ms: 100.0,
                max_memory_bytes: None,
            },
        );

        baselines
    }

    fn detect_regressions(&self) -> Vec<RegressionResult> {
        let mut regressions = Vec::new();

        for metric in &self.metrics {
            if let Some(baseline) = self.baselines.get(&metric.operation) {
                // Check speed regression
                if metric.ops_per_second < baseline.min_ops_per_second {
                    let ratio = metric.ops_per_second / baseline.min_ops_per_second;
                    let severity = if ratio < 0.5 {
                        RegressionSeverity::Critical
                    } else if ratio < 0.9 {
                        RegressionSeverity::Major
                    } else {
                        RegressionSeverity::Minor
                    };

                    regressions.push(RegressionResult {
                        operation: metric.operation.clone(),
                        baseline: baseline.clone(),
                        actual: metric.clone(),
                        regression_type: RegressionType::Speed,
                        severity,
                        message: format!(
                            "Performance regression: expected >= {} ops/sec, got {:.2} ops/sec ({:.1}% of baseline)",
                            baseline.min_ops_per_second,
                            metric.ops_per_second,
                            ratio * 100.0
                        ),
                    });
                }

                // Check memory regression if applicable
                if let (Some(baseline_mem), Some(actual_mem)) =
                    (baseline.max_memory_bytes, metric.memory_used_bytes)
                {
                    if actual_mem > baseline_mem {
                        let ratio = actual_mem as f64 / baseline_mem as f64;
                        let severity = if ratio > 2.0 {
                            RegressionSeverity::Critical
                        } else if ratio > 1.5 {
                            RegressionSeverity::Major
                        } else {
                            RegressionSeverity::Minor
                        };

                        regressions.push(RegressionResult {
                            operation: metric.operation.clone(),
                            baseline: baseline.clone(),
                            actual: metric.clone(),
                            regression_type: RegressionType::Memory,
                            severity,
                            message: format!(
                                "Memory regression: expected <= {} bytes, got {} bytes ({:.1}x baseline)",
                                baseline_mem,
                                actual_mem,
                                ratio
                            ),
                        });
                    }
                }
            }
        }

        regressions
    }

    fn create_report(&self) -> PerformanceReport {
        let regressions = self.detect_regressions();
        let critical_count = regressions
            .iter()
            .filter(|r| matches!(r.severity, RegressionSeverity::Critical))
            .count();

        let total_ops = self.metrics.len();
        let passed = total_ops - regressions.len();

        let avg_ratio = if !self.metrics.is_empty() {
            let sum: f64 = self
                .metrics
                .iter()
                .filter_map(|m| {
                    self.baselines
                        .get(&m.operation)
                        .map(|b| m.ops_per_second / b.min_ops_per_second)
                })
                .sum();
            let count = self
                .metrics
                .iter()
                .filter(|m| self.baselines.contains_key(&m.operation))
                .count();

            if count > 0 {
                sum / count as f64
            } else {
                1.0
            }
        } else {
            1.0
        };

        PerformanceReport {
            timestamp: (current_timestamp_ms() / 1000.0) as u64,
            environment: self.environment.clone(),
            metrics: self.metrics.clone(),
            regressions,
            summary: ReportSummary {
                total_operations: total_ops,
                passed,
                regressions: total_ops - passed,
                critical_regressions: critical_count,
                average_performance_ratio: avg_ratio,
            },
        }
    }
}

/// JavaScript-friendly performance report formatter
#[wasm_bindgen]
pub struct PerformanceReportFormatter;

#[wasm_bindgen]
impl PerformanceReportFormatter {
    /// Format a performance report as HTML
    pub fn format_html(report_json: &str) -> Result<String, JsValue> {
        let report: PerformanceReport = serde_json::from_str(report_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse report: {}", e)))?;

        let mut html = String::from(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>Performance Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .summary { background: #f0f0f0; padding: 15px; border-radius: 5px; margin-bottom: 20px; }
        .passed { color: green; }
        .failed { color: red; }
        .regression { background: #ffeeee; padding: 10px; margin: 10px 0; border-radius: 5px; }
        .critical { border-left: 5px solid red; }
        .major { border-left: 5px solid orange; }
        .minor { border-left: 5px solid yellow; }
        table { border-collapse: collapse; width: 100%; margin-top: 20px; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background: #f0f0f0; }
    </style>
</head>
<body>
    <h1>Performance Report</h1>
"#,
        );

        // Summary section
        html.push_str(&format!(
            r#"
    <div class="summary">
        <h2>Summary</h2>
        <p>Environment: {}</p>
        <p>Total Operations: {}</p>
        <p class="{}">Passed: {}</p>
        <p class="{}">Regressions: {} (Critical: {})</p>
        <p>Average Performance Ratio: {:.2}x baseline</p>
    </div>
"#,
            report.environment,
            report.summary.total_operations,
            if report.summary.regressions == 0 {
                "passed"
            } else {
                "failed"
            },
            report.summary.passed,
            if report.summary.regressions == 0 {
                "passed"
            } else {
                "failed"
            },
            report.summary.regressions,
            report.summary.critical_regressions,
            report.summary.average_performance_ratio
        ));

        // Regressions section
        if !report.regressions.is_empty() {
            html.push_str("<h2>Regressions</h2>");
            for regression in &report.regressions {
                let severity_class = match regression.severity {
                    RegressionSeverity::Critical => "critical",
                    RegressionSeverity::Major => "major",
                    RegressionSeverity::Minor => "minor",
                };

                html.push_str(&format!(
                    r#"
    <div class="regression {}">
        <h3>{}</h3>
        <p>{}</p>
        <p>Expected: ≥ {} ops/sec, Actual: {:.2} ops/sec</p>
    </div>
"#,
                    severity_class,
                    regression.operation,
                    regression.message,
                    regression.baseline.min_ops_per_second,
                    regression.actual.ops_per_second
                ));
            }
        }

        // Metrics table
        html.push_str(
            r#"
    <h2>All Metrics</h2>
    <table>
        <tr>
            <th>Operation</th>
            <th>Duration (ms)</th>
            <th>Iterations</th>
            <th>Ops/Second</th>
            <th>Memory (bytes)</th>
        </tr>
"#,
        );

        for metric in &report.metrics {
            html.push_str(&format!(
                r#"
        <tr>
            <td>{}</td>
            <td>{:.2}</td>
            <td>{}</td>
            <td>{:.0}</td>
            <td>{}</td>
        </tr>
"#,
                metric.operation,
                metric.duration_ms,
                metric.iterations,
                metric.ops_per_second,
                metric
                    .memory_used_bytes
                    .map(|b| b.to_string())
                    .unwrap_or_else(|| "N/A".to_string())
            ));
        }

        html.push_str(
            r#"
    </table>
</body>
</html>
"#,
        );

        Ok(html)
    }

    /// Format a performance report as Markdown
    pub fn format_markdown(report_json: &str) -> Result<String, JsValue> {
        let report: PerformanceReport = serde_json::from_str(report_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse report: {}", e)))?;

        let mut md = String::from("# Performance Report\n\n");

        // Summary
        md.push_str("## Summary\n\n");
        md.push_str(&format!("- **Environment**: {}\n", report.environment));
        md.push_str(&format!(
            "- **Total Operations**: {}\n",
            report.summary.total_operations
        ));
        md.push_str(&format!("- **Passed**: {}\n", report.summary.passed));
        md.push_str(&format!(
            "- **Regressions**: {} (Critical: {})\n",
            report.summary.regressions, report.summary.critical_regressions
        ));
        md.push_str(&format!(
            "- **Average Performance Ratio**: {:.2}x baseline\n\n",
            report.summary.average_performance_ratio
        ));

        // Regressions
        if !report.regressions.is_empty() {
            md.push_str("## Regressions\n\n");
            for regression in &report.regressions {
                md.push_str(&format!(
                    "### {} ({:?})\n\n{}\n\n",
                    regression.operation, regression.severity, regression.message
                ));
            }
        }

        // Metrics table
        md.push_str("## All Metrics\n\n");
        md.push_str("| Operation | Duration (ms) | Iterations | Ops/Second | Memory |\n");
        md.push_str("|-----------|---------------|------------|------------|--------|\n");

        for metric in &report.metrics {
            md.push_str(&format!(
                "| {} | {:.2} | {} | {:.0} | {} |\n",
                metric.operation,
                metric.duration_ms,
                metric.iterations,
                metric.ops_per_second,
                metric
                    .memory_used_bytes
                    .map(|b| format!("{} bytes", b))
                    .unwrap_or_else(|| "N/A".to_string())
            ));
        }

        Ok(md)
    }
}

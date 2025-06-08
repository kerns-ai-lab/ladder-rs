use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct BaselineData {
    timestamp: u64,
    version: String,
    measurements: HashMap<String, HashMap<String, BenchmarkResult>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkResult {
    mean_time_ns: f64,
    throughput_ops_per_sec: f64,
    std_dev_ns: Option<f64>,
}

#[derive(Debug)]
struct RegressionResult {
    benchmark_name: String,
    group_name: String,
    baseline_throughput: f64,
    current_throughput: f64,
    percentage_change: f64,
    is_regression: bool,
    severity: RegressionSeverity,
}

#[derive(Debug, PartialEq)]
enum RegressionSeverity {
    None,
    Minor,    // 5-10% regression
    Major,    // 10-20% regression
    Critical, // >20% regression
}

struct RegressionDetector {
    regression_threshold: f64, // 5% default
    major_threshold: f64,      // 10%
    critical_threshold: f64,   // 20%
}

impl RegressionDetector {
    fn new() -> Self {
        Self {
            regression_threshold: 0.05, // 5%
            major_threshold: 0.10,      // 10%
            critical_threshold: 0.20,   // 20%
        }
    }

    fn load_baseline(
        &self,
        baseline_path: &Path,
    ) -> Result<BaselineData, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(baseline_path)?;
        let baseline: BaselineData = serde_json::from_str(&content)?;
        Ok(baseline)
    }

    fn detect_regressions(
        &self,
        baseline: &BaselineData,
        current: &BaselineData,
    ) -> Vec<RegressionResult> {
        let mut regressions = Vec::new();

        for (group_name, current_group) in &current.measurements {
            if let Some(baseline_group) = baseline.measurements.get(group_name) {
                for (bench_name, current_result) in current_group {
                    if let Some(baseline_result) = baseline_group.get(bench_name) {
                        let result = self.compare_results(
                            group_name.clone(),
                            bench_name.clone(),
                            baseline_result,
                            current_result,
                        );
                        regressions.push(result);
                    }
                }
            }
        }

        regressions
    }

    fn compare_results(
        &self,
        group_name: String,
        bench_name: String,
        baseline: &BenchmarkResult,
        current: &BenchmarkResult,
    ) -> RegressionResult {
        // Calculate percentage change in throughput (higher is better)
        let percentage_change = if baseline.throughput_ops_per_sec > 0.0 {
            (current.throughput_ops_per_sec - baseline.throughput_ops_per_sec)
                / baseline.throughput_ops_per_sec
        } else {
            0.0
        };

        // Negative percentage means performance degradation
        let is_regression = percentage_change < -self.regression_threshold;

        let severity = if percentage_change < -self.critical_threshold {
            RegressionSeverity::Critical
        } else if percentage_change < -self.major_threshold {
            RegressionSeverity::Major
        } else if percentage_change < -self.regression_threshold {
            RegressionSeverity::Minor
        } else {
            RegressionSeverity::None
        };

        RegressionResult {
            benchmark_name: bench_name,
            group_name,
            baseline_throughput: baseline.throughput_ops_per_sec,
            current_throughput: current.throughput_ops_per_sec,
            percentage_change,
            is_regression,
            severity,
        }
    }

    fn generate_report(&self, regressions: &[RegressionResult]) -> String {
        let mut report = String::new();

        report.push_str("# Performance Regression Analysis Report\n\n");

        let total_benchmarks = regressions.len();
        let regressions_found: Vec<_> = regressions.iter().filter(|r| r.is_regression).collect();

        let critical_count = regressions_found
            .iter()
            .filter(|r| r.severity == RegressionSeverity::Critical)
            .count();
        let major_count = regressions_found
            .iter()
            .filter(|r| r.severity == RegressionSeverity::Major)
            .count();
        let minor_count = regressions_found
            .iter()
            .filter(|r| r.severity == RegressionSeverity::Minor)
            .count();

        // Summary
        report.push_str("## Summary\n\n");
        if regressions_found.is_empty() {
            report.push_str("✅ **No performance regressions detected**\n\n");
        } else {
            report.push_str(&format!(
                "⚠️ **{} performance regressions detected out of {} benchmarks**\n\n",
                regressions_found.len(),
                total_benchmarks
            ));

            if critical_count > 0 {
                report.push_str(&format!("🔴 Critical regressions: {}\n", critical_count));
            }
            if major_count > 0 {
                report.push_str(&format!("🟡 Major regressions: {}\n", major_count));
            }
            if minor_count > 0 {
                report.push_str(&format!("🟠 Minor regressions: {}\n", minor_count));
            }
            report.push('\n');
        }

        // Configuration
        report.push_str("## Configuration\n\n");
        report.push_str(&format!(
            "- Regression threshold: {:.1}%\n",
            self.regression_threshold * 100.0
        ));
        report.push_str(&format!(
            "- Major threshold: {:.1}%\n",
            self.major_threshold * 100.0
        ));
        report.push_str(&format!(
            "- Critical threshold: {:.1}%\n",
            self.critical_threshold * 100.0
        ));
        report.push('\n');

        // Detailed results
        if !regressions_found.is_empty() {
            report.push_str("## Detected Regressions\n\n");

            for regression in &regressions_found {
                let emoji = match regression.severity {
                    RegressionSeverity::Critical => "🔴",
                    RegressionSeverity::Major => "🟡",
                    RegressionSeverity::Minor => "🟠",
                    RegressionSeverity::None => "✅",
                };

                report.push_str(&format!(
                    "{} **{}/{}**\n",
                    emoji, regression.group_name, regression.benchmark_name
                ));
                report.push_str(&format!(
                    "- Baseline: {:.0} ops/sec\n",
                    regression.baseline_throughput
                ));
                report.push_str(&format!(
                    "- Current: {:.0} ops/sec\n",
                    regression.current_throughput
                ));
                report.push_str(&format!(
                    "- Change: {:.1}%\n",
                    regression.percentage_change * 100.0
                ));
                report.push('\n');
            }
        }

        // All results summary
        report.push_str("## All Benchmark Results\n\n");
        let mut grouped_results: HashMap<String, Vec<&RegressionResult>> = HashMap::new();
        for result in regressions {
            grouped_results
                .entry(result.group_name.clone())
                .or_default()
                .push(result);
        }

        for (group_name, group_results) in grouped_results {
            report.push_str(&format!(
                "### {}\n\n",
                group_name.replace('_', " ").to_uppercase()
            ));

            for result in group_results {
                let status = if result.percentage_change >= 0.0 {
                    "✅"
                } else if !result.is_regression {
                    "🟢"
                } else {
                    match result.severity {
                        RegressionSeverity::Critical => "🔴",
                        RegressionSeverity::Major => "🟡",
                        RegressionSeverity::Minor => "🟠",
                        RegressionSeverity::None => "✅",
                    }
                };

                report.push_str(&format!(
                    "{} **{}**: {:.0} ops/sec ({:+.1}%)\n",
                    status,
                    result.benchmark_name,
                    result.current_throughput,
                    result.percentage_change * 100.0
                ));
            }
            report.push('\n');
        }

        report
    }

    fn should_fail_ci(&self, regressions: &[RegressionResult]) -> bool {
        // Fail CI if there are any critical regressions or more than 3 major regressions
        let critical_count = regressions
            .iter()
            .filter(|r| r.severity == RegressionSeverity::Critical)
            .count();
        let major_count = regressions
            .iter()
            .filter(|r| r.severity == RegressionSeverity::Major)
            .count();

        critical_count > 0 || major_count > 3
    }

    fn run_analysis(
        &self,
        baseline_path: &Path,
        current_path: &Path,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        println!("🔍 Performance Regression Analysis");
        println!("{}", "=".repeat(40));

        // Load baselines
        println!("📊 Loading baseline data...");
        let baseline = self.load_baseline(baseline_path)?;
        let current = self.load_baseline(current_path)?;

        println!(
            "📈 Analyzing {} benchmark groups...",
            current.measurements.len()
        );

        // Detect regressions
        let regressions = self.detect_regressions(&baseline, &current);

        // Generate report
        let report = self.generate_report(&regressions);

        // Save report
        let report_path = Path::new("regression_analysis.md");
        fs::write(report_path, &report)?;
        println!("📋 Regression report saved to {}", report_path.display());

        // Print summary to console
        let regression_count = regressions.iter().filter(|r| r.is_regression).count();
        if regression_count == 0 {
            println!("✅ No performance regressions detected!");
        } else {
            println!("⚠️  {} performance regressions detected", regression_count);
        }

        // Determine if CI should fail
        let should_fail = self.should_fail_ci(&regressions);
        if should_fail {
            println!("❌ Critical regressions detected - CI should fail");
        } else {
            println!("✅ No critical regressions - CI can pass");
        }

        Ok(!should_fail) // Return true if CI should pass
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <baseline.json> <current.json>", args[0]);
        eprintln!("  baseline.json: Path to baseline performance data");
        eprintln!("  current.json:  Path to current performance data");
        std::process::exit(1);
    }

    let baseline_path = Path::new(&args[1]);
    let current_path = Path::new(&args[2]);

    if !baseline_path.exists() {
        eprintln!(
            "Error: Baseline file does not exist: {}",
            baseline_path.display()
        );
        std::process::exit(1);
    }

    if !current_path.exists() {
        eprintln!(
            "Error: Current file does not exist: {}",
            current_path.display()
        );
        std::process::exit(1);
    }

    let detector = RegressionDetector::new();
    match detector.run_analysis(baseline_path, current_path) {
        Ok(should_pass) => {
            if should_pass {
                println!("\n🎉 Regression analysis completed successfully!");
                std::process::exit(0);
            } else {
                println!("\n💥 Critical performance regressions detected!");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error during regression analysis: {}", e);
            std::process::exit(1);
        }
    }
}

use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct BaselineData {
    timestamp: u64,
    version: String,
    system_info: SystemInfo,
    performance_targets: HashMap<String, PerformanceTarget>,
    measurements: HashMap<String, HashMap<String, BenchmarkResult>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SystemInfo {
    platform: String,
    architecture: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PerformanceTarget {
    target_throughput_ops_per_sec: f64,
    max_latency_us: f64,
    target_single_game_us: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkResult {
    mean_time_ns: f64,
    mean_time_us: f64,
    mean_time_ms: f64,
    throughput_ops_per_sec: f64,
    std_dev_ns: Option<f64>,
    median_time_ns: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CriterionEstimates {
    mean: Option<CriterionPoint>,
    median: Option<CriterionPoint>,
    std_dev: Option<CriterionPoint>,
    slope: Option<CriterionPoint>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CriterionPoint {
    point_estimate: f64,
}

struct BaselineEstablisher {
    project_root: std::path::PathBuf,
    baselines_dir: std::path::PathBuf,
}

impl BaselineEstablisher {
    fn new() -> Self {
        let project_root = std::env::current_dir()
            .expect("Failed to get current directory");
        let baselines_dir = project_root.join("benchmarks").join("baselines");
        
        fs::create_dir_all(&baselines_dir)
            .expect("Failed to create baselines directory");
            
        Self {
            project_root,
            baselines_dir,
        }
    }

    fn run_benchmarks(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Running comprehensive benchmarks...");
        
        let benchmark_suites = vec![
            "rating_update",
            "algorithm_specific",
            "integration_scenarios",
        ];
        
        for suite in &benchmark_suites {
            println!("  📊 Running {} benchmark suite...", suite);
            
            let output = Command::new("cargo")
                .args(&["bench", "--bench", suite])
                .current_dir(&self.project_root)
                .output()?;
                
            if !output.status.success() {
                eprintln!("  ❌ {} benchmark failed:", suite);
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                return Err(format!("Benchmark {} failed", suite).into());
            } else {
                println!("  ✅ {} benchmark completed", suite);
            }
        }
        
        Ok(())
    }

    fn collect_criterion_results(&self) -> Result<HashMap<String, HashMap<String, BenchmarkResult>>, Box<dyn std::error::Error>> {
        println!("📈 Collecting benchmark results...");
        
        let criterion_dir = self.project_root.join("target").join("criterion");
        if !criterion_dir.exists() {
            return Err("No Criterion results found".into());
        }
        
        let mut results = HashMap::new();
        
        for entry in fs::read_dir(&criterion_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let bench_name = entry.file_name().to_string_lossy().to_string();
                if let Ok(bench_results) = self.parse_benchmark_dir(&entry.path()) {
                    if !bench_results.is_empty() {
                        results.insert(bench_name, bench_results);
                    }
                }
            }
        }
        
        println!("📊 Collected results for {} benchmark groups", results.len());
        Ok(results)
    }

    fn parse_benchmark_dir(&self, bench_dir: &Path) -> Result<HashMap<String, BenchmarkResult>, Box<dyn std::error::Error>> {
        let mut results = HashMap::new();
        
        for entry in fs::read_dir(bench_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let estimates_file = entry.path().join("base").join("estimates.json");
                if estimates_file.exists() {
                    if let Ok(estimates_content) = fs::read_to_string(&estimates_file) {
                        if let Ok(estimates) = serde_json::from_str::<CriterionEstimates>(&estimates_content) {
                            if let Some(mean) = estimates.mean {
                                let mean_ns = mean.point_estimate;
                                let mean_us = mean_ns / 1000.0;
                                let mean_ms = mean_us / 1000.0;
                                let ops_per_sec = if mean_ns > 0.0 { 1_000_000_000.0 / mean_ns } else { 0.0 };
                                
                                results.insert(entry.file_name().to_string_lossy().to_string(), BenchmarkResult {
                                    mean_time_ns: mean_ns,
                                    mean_time_us: mean_us,
                                    mean_time_ms: mean_ms,
                                    throughput_ops_per_sec: ops_per_sec,
                                    std_dev_ns: estimates.std_dev.map(|s| s.point_estimate),
                                    median_time_ns: estimates.median.map(|m| m.point_estimate),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(results)
    }

    fn get_version(&self) -> String {
        let cargo_toml_path = self.project_root.join("Cargo.toml");
        if let Ok(content) = fs::read_to_string(cargo_toml_path) {
            for line in content.lines() {
                if line.starts_with("version =") {
                    if let Some(version) = line.split('=').nth(1) {
                        return version.trim().trim_matches('"').to_string();
                    }
                }
            }
        }
        "unknown".to_string()
    }

    fn get_system_info(&self) -> SystemInfo {
        SystemInfo {
            platform: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
        }
    }

    fn get_performance_targets(&self) -> HashMap<String, PerformanceTarget> {
        let mut targets = HashMap::new();
        
        targets.insert("elo".to_string(), PerformanceTarget {
            target_throughput_ops_per_sec: 100_000.0,
            max_latency_us: 100.0,
            target_single_game_us: 10.0,
        });
        
        targets.insert("glicko".to_string(), PerformanceTarget {
            target_throughput_ops_per_sec: 50_000.0,
            max_latency_us: 200.0,
            target_single_game_us: 20.0,
        });
        
        targets.insert("trueskill_simplified".to_string(), PerformanceTarget {
            target_throughput_ops_per_sec: 25_000.0,
            max_latency_us: 400.0,
            target_single_game_us: 40.0,
        });
        
        targets.insert("trueskill_factor_graph".to_string(), PerformanceTarget {
            target_throughput_ops_per_sec: 5_000.0,
            max_latency_us: 2_000.0,
            target_single_game_us: 200.0,
        });
        
        targets
    }

    fn create_baseline(&self, measurements: HashMap<String, HashMap<String, BenchmarkResult>>) -> BaselineData {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
            
        BaselineData {
            timestamp,
            version: self.get_version(),
            system_info: self.get_system_info(),
            performance_targets: self.get_performance_targets(),
            measurements,
        }
    }

    fn save_baseline(&self, baseline: &BaselineData) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        let timestamp_str = chrono::DateTime::from_timestamp(baseline.timestamp as i64, 0)
            .unwrap_or_else(|| chrono::Utc::now())
            .format("%Y%m%d_%H%M%S")
            .to_string();
            
        let baseline_file = self.baselines_dir.join(format!("baseline_{}.json", timestamp_str));
        let current_baseline = self.baselines_dir.join("current_baseline.json");
        
        let baseline_json = serde_json::to_string_pretty(baseline)?;
        
        fs::write(&baseline_file, &baseline_json)?;
        fs::write(&current_baseline, &baseline_json)?;
        
        println!("💾 Baseline saved to {}", baseline_file.display());
        println!("💾 Current baseline updated at {}", current_baseline.display());
        
        Ok(baseline_file)
    }

    fn generate_baseline_report(&self, baseline: &BaselineData) -> String {
        let mut report = String::new();
        
        report.push_str("# Performance Baseline Report\n");
        report.push_str(&format!("Generated: {}\n", 
            chrono::DateTime::from_timestamp(baseline.timestamp as i64, 0)
                .unwrap_or_else(|| chrono::Utc::now())
                .format("%Y-%m-%d %H:%M:%S UTC")
        ));
        report.push_str(&format!("Version: {}\n\n", baseline.version));
        
        // System information
        report.push_str("## System Information\n");
        report.push_str(&format!("- **Platform**: {}\n", baseline.system_info.platform));
        report.push_str(&format!("- **Architecture**: {}\n\n", baseline.system_info.architecture));
        
        // Performance targets
        report.push_str("## Performance Targets\n");
        for (algo, targets) in &baseline.performance_targets {
            report.push_str(&format!("### {}\n", algo.replace('_', " ").to_uppercase()));
            report.push_str(&format!("- Target Throughput: {:.0} ops/sec\n", targets.target_throughput_ops_per_sec));
            report.push_str(&format!("- Max Latency: {:.1} μs\n", targets.max_latency_us));
            report.push_str(&format!("- Target Single Game: {:.1} μs\n\n", targets.target_single_game_us));
        }
        
        // Benchmark results summary
        report.push_str("## Benchmark Results Summary\n");
        for (group_name, group_data) in &baseline.measurements {
            report.push_str(&format!("### {}\n", group_name.replace('_', " ").to_uppercase()));
            
            for (bench_name, metrics) in group_data {
                report.push_str(&format!("- **{}**: {:.2} μs ({:.0} ops/sec)\n", 
                    bench_name, 
                    metrics.mean_time_us, 
                    metrics.throughput_ops_per_sec
                ));
            }
            report.push_str("\n");
        }
        
        // Performance analysis
        report.push_str("## Performance Analysis\n");
        report.push_str(&self.analyze_performance(baseline));
        
        report
    }

    fn analyze_performance(&self, baseline: &BaselineData) -> String {
        let mut analysis = String::new();
        
        for (algo, targets) in &baseline.performance_targets {
            let target_throughput = targets.target_throughput_ops_per_sec;
            let mut max_actual_throughput: f64 = 0.0;
            
            // Find the best throughput for this algorithm
            for (group_name, group_data) in &baseline.measurements {
                for (bench_name, metrics) in group_data {
                    if bench_name.to_lowercase().contains(&algo.to_lowercase()) {
                        max_actual_throughput = max_actual_throughput.max(metrics.throughput_ops_per_sec);
                    }
                }
            }
            
            if max_actual_throughput > 0.0 {
                let percentage = (max_actual_throughput / target_throughput) * 100.0;
                let status = if percentage >= 100.0 { "✅" } 
                           else if percentage >= 80.0 { "⚠️" } 
                           else { "❌" };
                           
                analysis.push_str(&format!("{} **{}**: {:.1}% of target ({:.0} / {:.0} ops/sec)\n",
                    status,
                    algo.replace('_', " ").to_uppercase(),
                    percentage,
                    max_actual_throughput,
                    target_throughput
                ));
            } else {
                analysis.push_str(&format!("❓ **{}**: No measurements found\n", 
                    algo.replace('_', " ").to_uppercase()));
            }
        }
        
        if analysis.is_empty() {
            analysis.push_str("No performance analysis available.\n");
        }
        
        analysis
    }

    fn establish_baseline(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🎯 Ladder-rs Performance Baseline Establishment");
        println!("{}", "=".repeat(50));
        
        // Step 1: Run benchmarks
        self.run_benchmarks()?;
        
        // Step 2: Collect results
        let measurements = self.collect_criterion_results()?;
        if measurements.is_empty() {
            return Err("No benchmark results found. Cannot establish baseline.".into());
        }
        
        // Step 3: Create baseline
        let baseline = self.create_baseline(measurements);
        
        // Step 4: Save baseline
        let baseline_file = self.save_baseline(&baseline)?;
        
        // Step 5: Generate report
        let report = self.generate_baseline_report(&baseline);
        let report_file = baseline_file.with_extension("md");
        fs::write(&report_file, report)?;
        
        println!("📋 Baseline report saved to {}", report_file.display());
        
        // Print summary
        println!("\n{}", "=".repeat(50));
        println!("✅ Performance baseline established successfully!");
        println!("📊 {} benchmark groups measured", baseline.measurements.len());
        println!("💾 Baseline saved to {}", baseline_file.display());
        println!("📋 Report available at {}", report_file.display());
        
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let establisher = BaselineEstablisher::new();
    establisher.establish_baseline()
}
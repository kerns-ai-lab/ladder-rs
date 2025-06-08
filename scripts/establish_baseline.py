#!/usr/bin/env python3
"""
Performance Baseline Establishment Script

This script runs comprehensive benchmarks and establishes performance baselines
for the ladder-rs rating system library. It generates baseline data that can be
used for regression detection in CI.
"""

import json
import os
import subprocess
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, Any, List
import statistics


class BaselineEstablisher:
    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.baselines_dir = project_root / "benchmarks" / "baselines"
        self.baselines_dir.mkdir(parents=True, exist_ok=True)
        
    def run_benchmarks(self) -> bool:
        """Run comprehensive benchmarks and collect results."""
        print("🚀 Running comprehensive benchmarks...")
        
        # List of benchmark suites to run
        benchmark_suites = [
            "rating_update",
            "algorithm_specific", 
            "integration_scenarios"
        ]
        
        success = True
        for suite in benchmark_suites:
            print(f"  📊 Running {suite} benchmark suite...")
            try:
                result = subprocess.run(
                    ["cargo", "bench", "--bench", suite],
                    cwd=self.project_root,
                    capture_output=True,
                    text=True,
                    timeout=300  # 5 minute timeout per suite
                )
                
                if result.returncode != 0:
                    print(f"  ❌ {suite} benchmark failed:")
                    print(result.stderr)
                    success = False
                else:
                    print(f"  ✅ {suite} benchmark completed")
                    
            except subprocess.TimeoutExpired:
                print(f"  ⏰ {suite} benchmark timed out")
                success = False
            except Exception as e:
                print(f"  ❌ Error running {suite} benchmark: {e}")
                success = False
                
        return success
    
    def collect_criterion_results(self) -> Dict[str, Any]:
        """Collect and parse Criterion benchmark results."""
        print("📈 Collecting benchmark results...")
        
        criterion_dir = self.project_root / "target" / "criterion"
        if not criterion_dir.exists():
            print("❌ No Criterion results found")
            return {}
            
        results = {}
        
        # Walk through all benchmark results
        for bench_dir in criterion_dir.iterdir():
            if not bench_dir.is_dir():
                continue
                
            bench_name = bench_dir.name
            bench_results = self._parse_benchmark_dir(bench_dir)
            
            if bench_results:
                results[bench_name] = bench_results
                
        print(f"📊 Collected results for {len(results)} benchmark groups")
        return results
    
    def _parse_benchmark_dir(self, bench_dir: Path) -> Dict[str, Any]:
        """Parse a single benchmark directory for results."""
        results = {}
        
        # Look for subdirectories (individual benchmarks)
        for item in bench_dir.iterdir():
            if not item.is_dir():
                continue
                
            estimates_file = item / "base" / "estimates.json"
            if estimates_file.exists():
                try:
                    with open(estimates_file, 'r') as f:
                        estimates = json.load(f)
                    
                    # Extract key performance metrics
                    results[item.name] = {
                        "mean_estimate": estimates.get("mean", {}).get("point_estimate"),
                        "median_estimate": estimates.get("median", {}).get("point_estimate"),
                        "std_dev": estimates.get("std_dev", {}).get("point_estimate"),
                        "slope": estimates.get("slope", {}).get("point_estimate"),
                    }
                    
                except (json.JSONDecodeError, FileNotFoundError) as e:
                    print(f"  ⚠️  Could not parse {estimates_file}: {e}")
                    
        return results
    
    def calculate_baseline_metrics(self, results: Dict[str, Any]) -> Dict[str, Any]:
        """Calculate baseline performance metrics from benchmark results."""
        print("🧮 Calculating baseline performance metrics...")
        
        baseline = {
            "timestamp": datetime.now().isoformat(),
            "version": self._get_version(),
            "system_info": self._get_system_info(),
            "performance_targets": self._get_performance_targets(),
            "measurements": {}
        }
        
        # Process each benchmark group
        for group_name, group_results in results.items():
            group_metrics = {}
            
            for bench_name, bench_data in group_results.items():
                if bench_data.get("mean_estimate"):
                    # Convert nanoseconds to more readable units
                    mean_ns = bench_data["mean_estimate"]
                    mean_us = mean_ns / 1000.0
                    mean_ms = mean_us / 1000.0
                    
                    # Calculate throughput (operations per second)
                    ops_per_sec = 1_000_000_000 / mean_ns if mean_ns > 0 else 0
                    
                    group_metrics[bench_name] = {
                        "mean_time_ns": mean_ns,
                        "mean_time_us": mean_us,
                        "mean_time_ms": mean_ms,
                        "throughput_ops_per_sec": ops_per_sec,
                        "std_dev_ns": bench_data.get("std_dev"),
                        "median_time_ns": bench_data.get("median_estimate")
                    }
            
            if group_metrics:
                baseline["measurements"][group_name] = group_metrics
                
        return baseline
    
    def _get_version(self) -> str:
        """Get the current version from Cargo.toml."""
        try:
            cargo_toml = self.project_root / "Cargo.toml"
            with open(cargo_toml, 'r') as f:
                for line in f:
                    if line.startswith('version ='):
                        return line.split('=')[1].strip().strip('"')
        except Exception:
            pass
        return "unknown"
    
    def _get_system_info(self) -> Dict[str, str]:
        """Get system information for baseline context."""
        try:
            import platform
            return {
                "platform": platform.platform(),
                "processor": platform.processor(),
                "python_version": platform.python_version(),
                "architecture": platform.architecture()[0]
            }
        except Exception:
            return {"platform": "unknown"}
    
    def _get_performance_targets(self) -> Dict[str, Dict[str, float]]:
        """Define performance targets for each algorithm."""
        return {
            "elo": {
                "target_throughput_ops_per_sec": 100000,
                "max_latency_us": 100,
                "target_single_game_us": 10
            },
            "glicko": {
                "target_throughput_ops_per_sec": 50000,
                "max_latency_us": 200,
                "target_single_game_us": 20
            },
            "trueskill_simplified": {
                "target_throughput_ops_per_sec": 25000,
                "max_latency_us": 400,
                "target_single_game_us": 40
            },
            "trueskill_factor_graph": {
                "target_throughput_ops_per_sec": 5000,
                "max_latency_us": 2000,
                "target_single_game_us": 200
            }
        }
    
    def save_baseline(self, baseline: Dict[str, Any]) -> Path:
        """Save baseline data to JSON file."""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        baseline_file = self.baselines_dir / f"baseline_{timestamp}.json"
        
        with open(baseline_file, 'w') as f:
            json.dump(baseline, f, indent=2)
            
        # Also save as current baseline
        current_baseline = self.baselines_dir / "current_baseline.json"
        with open(current_baseline, 'w') as f:
            json.dump(baseline, f, indent=2)
            
        print(f"💾 Baseline saved to {baseline_file}")
        print(f"💾 Current baseline updated at {current_baseline}")
        
        return baseline_file
    
    def generate_baseline_report(self, baseline: Dict[str, Any]) -> str:
        """Generate a human-readable baseline report."""
        report = []
        report.append("# Performance Baseline Report")
        report.append(f"Generated: {baseline['timestamp']}")
        report.append(f"Version: {baseline['version']}")
        report.append("")
        
        # System information
        report.append("## System Information")
        for key, value in baseline['system_info'].items():
            report.append(f"- **{key.title()}**: {value}")
        report.append("")
        
        # Performance targets
        report.append("## Performance Targets")
        for algo, targets in baseline['performance_targets'].items():
            report.append(f"### {algo.replace('_', ' ').title()}")
            for metric, value in targets.items():
                if 'ops_per_sec' in metric:
                    report.append(f"- {metric.replace('_', ' ').title()}: {value:,.0f}")
                else:
                    report.append(f"- {metric.replace('_', ' ').title()}: {value:.1f} μs")
            report.append("")
        
        # Benchmark results summary
        report.append("## Benchmark Results Summary")
        
        if 'measurements' in baseline:
            for group_name, group_data in baseline['measurements'].items():
                report.append(f"### {group_name.replace('_', ' ').title()}")
                
                for bench_name, metrics in group_data.items():
                    throughput = metrics.get('throughput_ops_per_sec', 0)
                    mean_us = metrics.get('mean_time_us', 0)
                    
                    report.append(f"- **{bench_name}**: {mean_us:.2f} μs ({throughput:,.0f} ops/sec)")
                
                report.append("")
        
        # Performance analysis
        report.append("## Performance Analysis")
        report.append(self._analyze_performance(baseline))
        
        return "\n".join(report)
    
    def _analyze_performance(self, baseline: Dict[str, Any]) -> str:
        """Analyze performance against targets."""
        analysis = []
        targets = baseline.get('performance_targets', {})
        measurements = baseline.get('measurements', {})
        
        # Check each algorithm against its targets
        for algo, algo_targets in targets.items():
            target_throughput = algo_targets.get('target_throughput_ops_per_sec', 0)
            
            # Find corresponding measurements
            actual_throughput = 0
            for group_name, group_data in measurements.items():
                for bench_name, metrics in group_data.items():
                    if algo in bench_name.lower():
                        actual_throughput = max(actual_throughput, metrics.get('throughput_ops_per_sec', 0))
            
            if actual_throughput > 0:
                percentage = (actual_throughput / target_throughput) * 100
                status = "✅" if percentage >= 100 else "⚠️" if percentage >= 80 else "❌"
                analysis.append(f"{status} **{algo.replace('_', ' ').title()}**: {percentage:.1f}% of target ({actual_throughput:,.0f} / {target_throughput:,.0f} ops/sec)")
            else:
                analysis.append(f"❓ **{algo.replace('_', ' ').title()}**: No measurements found")
        
        if not analysis:
            analysis.append("No performance analysis available.")
            
        return "\n".join(analysis)


def main():
    """Main entry point for baseline establishment."""
    project_root = Path(__file__).parent.parent
    
    print("🎯 Ladder-rs Performance Baseline Establishment")
    print("=" * 50)
    
    establisher = BaselineEstablisher(project_root)
    
    # Step 1: Run benchmarks
    if not establisher.run_benchmarks():
        print("❌ Benchmarks failed. Cannot establish baseline.")
        sys.exit(1)
    
    # Step 2: Collect results
    results = establisher.collect_criterion_results()
    if not results:
        print("❌ No benchmark results found. Cannot establish baseline.")
        sys.exit(1)
    
    # Step 3: Calculate baseline metrics
    baseline = establisher.calculate_baseline_metrics(results)
    
    # Step 4: Save baseline
    baseline_file = establisher.save_baseline(baseline)
    
    # Step 5: Generate report
    report = establisher.generate_baseline_report(baseline)
    report_file = baseline_file.with_suffix('.md')
    
    with open(report_file, 'w') as f:
        f.write(report)
        
    print(f"📋 Baseline report saved to {report_file}")
    
    # Print summary
    print("\n" + "=" * 50)
    print("✅ Performance baseline established successfully!")
    print(f"📊 {len(results)} benchmark groups measured")
    print(f"💾 Baseline saved to {baseline_file}")
    print(f"📋 Report available at {report_file}")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
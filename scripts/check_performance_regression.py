#!/usr/bin/env python3
"""
Performance regression detection script for ladder-rs.

This script analyzes benchmark results and detects performance regressions
by comparing current results against a baseline.
"""

import argparse
import json
import sys
from typing import Dict, List, Tuple


class PerformanceRegression:
    """Represents a detected performance regression."""
    
    def __init__(self, benchmark: str, baseline_value: float, current_value: float, 
                 metric: str, regression_percent: float):
        self.benchmark = benchmark
        self.baseline_value = baseline_value
        self.current_value = current_value
        self.metric = metric
        self.regression_percent = regression_percent
    
    def __str__(self):
        return (f"{self.benchmark}: {self.regression_percent:.1f}% regression "
                f"({self.baseline_value:.2f} -> {self.current_value:.2f} {self.metric})")


def load_criterion_results(filepath: str) -> Dict[str, Dict[str, float]]:
    """Load and parse Criterion benchmark results."""
    results = {}
    
    try:
        with open(filepath, 'r') as f:
            data = json.load(f)
        
        # Parse Criterion's JSON format
        if isinstance(data, list):
            for entry in data:
                if entry.get('reason') == 'benchmark-complete':
                    benchmark_id = entry.get('id', '')
                    if 'mean' in entry:
                        mean_estimate = entry['mean']['estimate']
                        results[benchmark_id] = {
                            'time_ns': mean_estimate,
                            'time_ms': mean_estimate / 1_000_000,
                            'throughput': entry.get('throughput', {}).get('per_iteration', 0)
                        }
    except Exception as e:
        print(f"Error loading {filepath}: {e}", file=sys.stderr)
    
    return results


def detect_regressions(baseline: Dict[str, Dict[str, float]], 
                      current: Dict[str, Dict[str, float]], 
                      threshold: float) -> List[PerformanceRegression]:
    """Detect performance regressions above the threshold."""
    regressions = []
    
    for benchmark_name, baseline_metrics in baseline.items():
        if benchmark_name not in current:
            print(f"Warning: Benchmark '{benchmark_name}' not found in current results")
            continue
        
        current_metrics = current[benchmark_name]
        
        # Check time regression (higher is worse)
        if baseline_metrics['time_ns'] > 0:
            time_increase = ((current_metrics['time_ns'] - baseline_metrics['time_ns']) / 
                           baseline_metrics['time_ns']) * 100
            
            if time_increase > threshold:
                regressions.append(PerformanceRegression(
                    benchmark_name,
                    baseline_metrics['time_ms'],
                    current_metrics['time_ms'],
                    'ms',
                    time_increase
                ))
        
        # Check throughput regression (lower is worse)
        if baseline_metrics.get('throughput', 0) > 0:
            throughput_decrease = ((baseline_metrics['throughput'] - current_metrics.get('throughput', 0)) / 
                                 baseline_metrics['throughput']) * 100
            
            if throughput_decrease > threshold:
                regressions.append(PerformanceRegression(
                    benchmark_name,
                    baseline_metrics['throughput'],
                    current_metrics.get('throughput', 0),
                    'ops/sec',
                    throughput_decrease
                ))
    
    return regressions


def generate_markdown_report(regressions: List[PerformanceRegression], 
                           improvements: List[Tuple[str, float]]) -> str:
    """Generate a Markdown report of the performance analysis."""
    report = ["# Performance Analysis Report\n"]
    
    if not regressions:
        report.append("✅ **No performance regressions detected!**\n")
    else:
        report.append(f"⚠️ **{len(regressions)} performance regressions detected:**\n")
        report.append("| Benchmark | Baseline | Current | Regression |")
        report.append("|-----------|----------|---------|------------|")
        
        for reg in sorted(regressions, key=lambda r: r.regression_percent, reverse=True):
            report.append(f"| {reg.benchmark} | {reg.baseline_value:.2f} {reg.metric} | "
                         f"{reg.current_value:.2f} {reg.metric} | {reg.regression_percent:.1f}% |")
    
    if improvements:
        report.append("\n## Improvements\n")
        report.append("| Benchmark | Improvement |")
        report.append("|-----------|-------------|")
        
        for bench, improvement in sorted(improvements, key=lambda x: x[1], reverse=True)[:10]:
            report.append(f"| {bench} | {improvement:.1f}% faster |")
    
    return '\n'.join(report)


def main():
    parser = argparse.ArgumentParser(description='Check for performance regressions')
    parser.add_argument('--current', required=True, help='Current benchmark results JSON file')
    parser.add_argument('--baseline', required=True, help='Baseline benchmark results JSON file')
    parser.add_argument('--threshold', type=float, default=10.0, 
                       help='Regression threshold percentage (default: 10%)')
    parser.add_argument('--output', help='Output file for the report (default: stdout)')
    
    args = parser.parse_args()
    
    # Load benchmark results
    baseline_results = load_criterion_results(args.baseline)
    current_results = load_criterion_results(args.current)
    
    if not baseline_results:
        print("Error: No baseline results found", file=sys.stderr)
        sys.exit(1)
    
    if not current_results:
        print("Error: No current results found", file=sys.stderr)
        sys.exit(1)
    
    # Detect regressions
    regressions = detect_regressions(baseline_results, current_results, args.threshold)
    
    # Find improvements (optional)
    improvements = []
    for benchmark_name, baseline_metrics in baseline_results.items():
        if benchmark_name in current_results:
            current_metrics = current_results[benchmark_name]
            if baseline_metrics['time_ns'] > 0:
                time_decrease = ((baseline_metrics['time_ns'] - current_metrics['time_ns']) / 
                               baseline_metrics['time_ns']) * 100
                if time_decrease > args.threshold:
                    improvements.append((benchmark_name, time_decrease))
    
    # Generate report
    report = generate_markdown_report(regressions, improvements)
    
    if args.output:
        with open(args.output, 'w') as f:
            f.write(report)
    else:
        print(report)
    
    # Exit with error code if regressions found
    if regressions:
        print(f"\n❌ Found {len(regressions)} performance regressions above {args.threshold}% threshold", 
              file=sys.stderr)
        sys.exit(1)
    else:
        print(f"\n✅ No performance regressions above {args.threshold}% threshold", file=sys.stderr)
        sys.exit(0)


if __name__ == '__main__':
    main()
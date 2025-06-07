#!/bin/bash

# Script to generate comprehensive code coverage reports for ladder-rs

echo "Generating code coverage reports..."

# Generate HTML report
echo "Generating HTML coverage report..."
cargo llvm-cov test --html

# Generate JSON report  
echo "Generating JSON coverage report..."
cargo llvm-cov --json --output-path coverage.json

# Generate LCOV report
echo "Generating LCOV coverage report..."
cargo llvm-cov --lcov --output-path coverage.lcov

# Display summary
echo "Coverage Summary:"
cargo llvm-cov --summary-only

echo ""
echo "Coverage reports generated:"
echo "  - HTML: target/llvm-cov/html/index.html"
echo "  - JSON: coverage.json"
echo "  - LCOV: coverage.lcov"
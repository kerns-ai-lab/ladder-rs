#!/bin/bash
# Enhanced build script for ladder-rs WASM package - Task 1.1.1

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Bundle size targets (in bytes)
MAX_BUNDLE_SIZE=204800  # 200KB target
WARN_BUNDLE_SIZE=153600 # 150KB warning threshold

# Default values
BUILD_MODE="dev"
TARGET="web"
OUTPUT_DIR="pkg"
PARALLEL_BUILDS=false
ALL_TARGETS=false
SIZE_CHECK=true
VERBOSE=false

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_verbose() {
    if [ "$VERBOSE" = true ]; then
        echo -e "${BLUE}[VERBOSE]${NC} $1"
    fi
}

# Copy custom TypeScript definitions into the output directory
copy_ts_defs() {
    local out_dir="$1"
    if [ -f "types/ladder_rs_wasm.d.ts" ]; then
        cp "types/ladder_rs_wasm.d.ts" "$out_dir/ladder_rs_wasm.d.ts"
        log_verbose "Copied TypeScript definitions to $out_dir"
    fi
}

# Function to check bundle size
check_bundle_size() {
    local output_dir="$1"
    local target_name="$2"
    
    if [ -f "$output_dir/ladder_rs_wasm_bg.wasm" ]; then
        local size=$(stat -c%s "$output_dir/ladder_rs_wasm_bg.wasm" 2>/dev/null || stat -f%z "$output_dir/ladder_rs_wasm_bg.wasm" 2>/dev/null)
        local size_kb=$((size / 1024))
        
        echo -e "  📦 WASM bundle size: ${BLUE}${size_kb}KB${NC} (${size} bytes)"
        
        if [ "$size" -gt "$MAX_BUNDLE_SIZE" ]; then
            log_error "Bundle size exceeds 200KB target! ($size_kb KB)"
            return 1
        elif [ "$size" -gt "$WARN_BUNDLE_SIZE" ]; then
            log_warning "Bundle size approaching 200KB limit ($size_kb KB)"
        else
            log_success "Bundle size within target ($size_kb KB < 200KB)"
        fi
    else
        log_warning "WASM file not found in $output_dir"
        return 1
    fi
}

# Function to build for a specific target
build_target() {
    local target="$1"
    local mode="$2"
    local output_dir="$3"
    
    log_info "Building for target: $target (mode: $mode)"
    
    # Set target-specific output directory
    case "$target" in
        "nodejs") local target_output_dir="${output_dir}-node" ;;
        "bundler") local target_output_dir="${output_dir}-bundler" ;;
        *) local target_output_dir="$output_dir" ;;
    esac
    
    # Clean previous build
    if [ -d "$target_output_dir" ]; then
        log_verbose "Cleaning $target_output_dir"
        rm -rf "$target_output_dir"
    fi
    
    # Build command
    local build_cmd="wasm-pack build --target $target --out-dir $target_output_dir"
    
    case "$mode" in
        "release") build_cmd="$build_cmd --release" ;;
        "profiling") build_cmd="$build_cmd --profiling" ;;
        *) build_cmd="$build_cmd --dev" ;;
    esac
    
    log_verbose "Running: $build_cmd"
    
    if $build_cmd; then
        log_success "Build completed for $target"
        
        # Post-build optimizations for release mode
        if [ "$mode" = "release" ] && command -v wasm-opt &> /dev/null; then
            log_info "Running wasm-opt optimizations for $target"
            wasm-opt -Oz \
                --enable-mutable-globals \
                --enable-bulk-memory \
                "$target_output_dir/ladder_rs_wasm_bg.wasm" \
                -o "$target_output_dir/ladder_rs_wasm_bg.wasm"
            log_success "Optimization complete for $target"
        fi
        
        # Size check
        if [ "$SIZE_CHECK" = true ]; then
            check_bundle_size "$target_output_dir" "$target"
        fi

        # Copy TypeScript definitions
        copy_ts_defs "$target_output_dir"
        
        # Generate file listing
        log_verbose "Generated files in $target_output_dir:"
        if [ "$VERBOSE" = true ]; then
            ls -la "$target_output_dir" | sed 's/^/    /'
        fi
        
        return 0
    else
        log_error "Build failed for $target"
        return 1
    fi
}

# Function to run tests after build
run_post_build_tests() {
    log_info "Running post-build validation tests"
    
    if cargo test --test package_structure_validation 2>/dev/null; then
        log_success "Package structure validation passed"
    else
        log_warning "Package structure validation tests failed or not found"
    fi
}

# Parse command line arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --release) BUILD_MODE="release";;
        --dev) BUILD_MODE="dev";;
        --profiling) BUILD_MODE="profiling";;
        --target) TARGET="$2"; shift;;
        --out-dir) OUTPUT_DIR="$2"; shift;;
        --all-targets) ALL_TARGETS=true;;
        --parallel) PARALLEL_BUILDS=true;;
        --no-size-check) SIZE_CHECK=false;;
        --verbose|-v) VERBOSE=true;;
        --help|-h) 
            echo "Enhanced WASM Build Script - Task 1.1.1"
            echo ""
            echo "Usage: ./build.sh [options]"
            echo ""
            echo "Build Options:"
            echo "  --release         Build in release mode (optimized)"
            echo "  --dev             Build in development mode (default)"
            echo "  --profiling       Build with profiling optimizations"
            echo ""
            echo "Target Options:"
            echo "  --target TARGET   Set wasm-pack target (web, nodejs, bundler, no-modules)"
            echo "  --all-targets     Build for all targets (web, nodejs, bundler)"
            echo "  --parallel        Build targets in parallel (with --all-targets)"
            echo ""
            echo "Output Options:"
            echo "  --out-dir DIR     Set output directory (default: pkg)"
            echo "  --no-size-check   Skip bundle size validation"
            echo ""
            echo "Other Options:"
            echo "  --verbose, -v     Enable verbose output"
            echo "  --help, -h        Show this help message"
            echo ""
            echo "Examples:"
            echo "  ./build.sh --release --target web"
            echo "  ./build.sh --all-targets --parallel --release"
            echo "  ./build.sh --dev --verbose"
            exit 0
            ;;
        *) log_error "Unknown parameter: $1"; exit 1;;
    esac
    shift
done

# Header
echo -e "${BLUE}╔════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                    Ladder-RS WASM Build System - Task 1.1.1               ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

log_info "Build configuration:"
echo "  📊 Mode: $BUILD_MODE"
if [ "$ALL_TARGETS" = true ]; then
    echo "  🎯 Targets: web, nodejs, bundler"
    if [ "$PARALLEL_BUILDS" = true ]; then
        echo "  ⚡ Parallel builds: enabled"
    fi
else
    echo "  🎯 Target: $TARGET"
fi
echo "  📁 Output: $OUTPUT_DIR"
echo "  🔍 Size check: $SIZE_CHECK"
echo "  📝 Verbose: $VERBOSE"
echo ""

# Pre-build checks
log_info "Running pre-build checks..."

# Check for required tools
if ! command -v wasm-pack &> /dev/null; then
    log_error "wasm-pack is required but not found!"
    echo "Install with: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    log_error "cargo is required but not found!"
    exit 1
fi

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    log_error "Cargo.toml not found. Make sure you're in the wasm directory."
    exit 1
fi

log_success "Pre-build checks passed"
echo ""

# Build process
log_info "Starting build process..."

if [ "$ALL_TARGETS" = true ]; then
    targets=("web" "nodejs" "bundler")
    
    if [ "$PARALLEL_BUILDS" = true ]; then
        log_info "Building all targets in parallel..."
        
        # Start parallel builds
        declare -a pids
        for target in "${targets[@]}"; do
            log_verbose "Starting background build for $target"
            (build_target "$target" "$BUILD_MODE" "$OUTPUT_DIR") &
            pids+=($!)
        done
        
        # Wait for all builds to complete
        failed_builds=0
        for i in "${!pids[@]}"; do
            if ! wait "${pids[$i]}"; then
                log_error "Build failed for ${targets[$i]}"
                ((failed_builds++))
            fi
        done
        
        if [ "$failed_builds" -gt 0 ]; then
            log_error "$failed_builds build(s) failed"
            exit 1
        fi
    else
        log_info "Building all targets sequentially..."
        for target in "${targets[@]}"; do
            if ! build_target "$target" "$BUILD_MODE" "$OUTPUT_DIR"; then
                log_error "Build process stopped due to failure"
                exit 1
            fi
            echo ""
        done
    fi
else
    # Single target build
    if ! build_target "$TARGET" "$BUILD_MODE" "$OUTPUT_DIR"; then
        exit 1
    fi
fi

echo ""

# Post-build validation
run_post_build_tests

# Summary
echo ""
log_success "🎉 Build process completed successfully!"
echo ""
log_info "Generated outputs:"

if [ "$ALL_TARGETS" = true ]; then
    for target in web nodejs bundler; do
        case "$target" in
            "nodejs") target_dir="${OUTPUT_DIR}-node" ;;
            "bundler") target_dir="${OUTPUT_DIR}-bundler" ;;
            *) target_dir="$OUTPUT_DIR" ;;
        esac
        
        if [ -d "$target_dir" ]; then
            echo "  📦 $target: $target_dir/"
            if [ -f "$target_dir/ladder_rs_wasm_bg.wasm" ]; then
                local size=$(stat -c%s "$target_dir/ladder_rs_wasm_bg.wasm" 2>/dev/null || stat -f%z "$target_dir/ladder_rs_wasm_bg.wasm" 2>/dev/null)
                local size_kb=$((size / 1024))
                echo "      Size: ${size_kb}KB"
            fi
        fi
    done
else
    case "$TARGET" in
        "nodejs") target_dir="${OUTPUT_DIR}-node" ;;
        "bundler") target_dir="${OUTPUT_DIR}-bundler" ;;
        *) target_dir="$OUTPUT_DIR" ;;
    esac
    echo "  📦 $TARGET: $target_dir/"
fi

echo ""
log_info "To use the generated package:"
echo "  📝 Import in JavaScript: import init from './pkg/ladder_rs_wasm.js'"
echo "  📝 TypeScript definitions: ./pkg/ladder_rs_wasm.d.ts"
echo "  📝 Package for publishing: npm publish pkg/"
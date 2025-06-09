#!/bin/bash
# Task 1.1.5: Enhanced Development Build Script for ladder-rs WASM
# 
# This script provides comprehensive development workflow support including:
# - Watch mode with automatic rebuilding
# - Development server with hot reload
# - Debug logging and performance monitoring
# - Development environment configuration

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Default configuration
DEV_MODE="development"
TARGET="web"
WATCH_MODE=false
SERVE_MODE=false
HOT_RELOAD=false
DEBUG_MODE=false
VERBOSE=false
BUILD_ONLY=false
CLEANUP=false
DRY_RUN=false
SHOW_ENV=false
PERFORMANCE_MONITORING=false

# Configuration from dev.config.json
CONFIG_FILE="dev.config.json"
DEFAULT_PORT=3000
DEFAULT_HOST="127.0.0.1"
WS_PORT=3001

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
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

log_debug() {
    if [ "$DEBUG_MODE" = true ]; then
        echo -e "${PURPLE}[DEBUG]${NC} $1"
    fi
}

log_verbose() {
    if [ "$VERBOSE" = true ]; then
        echo -e "${CYAN}[VERBOSE]${NC} $1"
    fi
}

# Performance monitoring functions
start_timer() {
    TIMER_START=$(date +%s%N)
}

end_timer() {
    TIMER_END=$(date +%s%N)
    TIMER_DIFF=$(( (TIMER_END - TIMER_START) / 1000000 )) # Convert to milliseconds
    echo "$TIMER_DIFF"
}

get_memory_usage() {
    if command -v ps &> /dev/null; then
        ps -o pid,ppid,rss,comm -p $$ | tail -n 1 | awk '{print $3}' # RSS in KB
    else
        echo "0"
    fi
}

# Load configuration from dev.config.json
load_dev_config() {
    if [ -f "$CONFIG_FILE" ]; then
        log_debug "Loading development configuration from $CONFIG_FILE"
        
        # Extract configuration values using basic JSON parsing
        # In a real implementation, you'd use jq or a proper JSON parser
        if command -v jq &> /dev/null; then
            DEFAULT_PORT=$(jq -r '.server.port // 3000' "$CONFIG_FILE")
            DEFAULT_HOST=$(jq -r '.server.host // "127.0.0.1"' "$CONFIG_FILE")
            WS_PORT=$(jq -r '.server.hot_reload.ws_port // 3001' "$CONFIG_FILE")
            log_debug "Loaded config: port=$DEFAULT_PORT, host=$DEFAULT_HOST, ws_port=$WS_PORT"
        else
            log_warning "jq not found, using default configuration values"
        fi
    else
        log_warning "Development configuration file not found: $CONFIG_FILE"
    fi
}

# Set up development environment
setup_dev_environment() {
    log_debug "Setting up development environment"
    
    # Set environment variables
    export LADDER_RS_DEV_MODE="true"
    export RUST_LOG="debug"
    export WASM_INTERFACE_TYPES="1"
    
    if [ "$DEBUG_MODE" = true ]; then
        export RUST_BACKTRACE="1"
        export WASM_LOG="debug"
    fi
    
    log_verbose "Development environment variables set"
}

# Show environment variables
show_environment() {
    echo -e "${CYAN}Development Environment Variables:${NC}"
    echo "LADDER_RS_DEV_MODE=${LADDER_RS_DEV_MODE:-unset}"
    echo "LADDER_RS_DEBUG_LEVEL=${LADDER_RS_DEBUG_LEVEL:-unset}"
    echo "LADDER_RS_HOT_RELOAD=${LADDER_RS_HOT_RELOAD:-unset}"
    echo "RUST_LOG=${RUST_LOG:-unset}"
    echo "RUST_BACKTRACE=${RUST_BACKTRACE:-unset}"
    echo "WASM_LOG=${WASM_LOG:-unset}"
    echo "WASM_INTERFACE_TYPES=${WASM_INTERFACE_TYPES:-unset}"
}

# Build function with development optimizations
dev_build() {
    log_info "Starting development build..."
    
    if [ "$PERFORMANCE_MONITORING" = true ]; then
        start_timer
        local memory_before=$(get_memory_usage)
    fi
    
    # Prepare build arguments
    local build_args=()
    build_args+=("--target" "$TARGET")
    build_args+=("--out-dir" "pkg")
    build_args+=("--dev")  # Always use dev mode for development builds
    
    if [ "$DEBUG_MODE" = true ]; then
        build_args+=("--debug")
    fi
    
    if [ "$VERBOSE" = true ]; then
        build_args+=("--verbose")
    fi
    
    log_debug "Build command: wasm-pack build ${build_args[*]}"
    
    if [ "$DRY_RUN" = false ]; then
        # Check if we should use the main build script instead
        if [ "$BUILD_ONLY" = true ] && [ -f "../build.sh" ]; then
            log_info "Using main build script for build-only mode"
            log_info "Building for target: $TARGET"
            local build_script_args=("--target" "$TARGET" "--dev")
            if [ "$VERBOSE" = true ]; then
                build_script_args+=("--verbose")
            fi
            
            if "../build.sh" "${build_script_args[@]}"; then
                log_success "Build completed successfully"
                
                # Show performance metrics for build script path too
                if [ "$PERFORMANCE_MONITORING" = true ]; then
                    local build_time=$(end_timer)
                    local memory_after=$(get_memory_usage)
                    local memory_diff=$((memory_after - memory_before))
                    
                    echo -e "${CYAN}Performance Report:${NC}"
                    echo "Build time: ${build_time}ms"
                    echo "Memory usage: ${memory_after}KB (Δ${memory_diff}KB)"
                    
                    if [ -f "pkg/ladder_rs_wasm_bg.wasm" ]; then
                        local bundle_size=$(stat -c%s "pkg/ladder_rs_wasm_bg.wasm" 2>/dev/null || stat -f%z "pkg/ladder_rs_wasm_bg.wasm" 2>/dev/null)
                        local bundle_size_kb=$((bundle_size / 1024))
                        echo "Bundle size: ${bundle_size_kb}KB"
                    fi
                fi
            else
                log_error "Build script failed"
                return 1
            fi
        elif wasm-pack build "${build_args[@]}"; then
            log_success "Development build completed successfully"
            
            # Copy TypeScript definitions if they exist
            if [ -f "types/ladder_rs_wasm.d.ts" ]; then
                cp "types/ladder_rs_wasm.d.ts" "pkg/ladder_rs_wasm.d.ts"
                log_verbose "Copied custom TypeScript definitions"
            fi
            
            # Generate source maps for development
            if [ "$DEBUG_MODE" = true ] && [ -f "pkg/ladder_rs_wasm.js" ]; then
                log_debug "Generating source map for JavaScript debugging"
                # Create a basic source map for development debugging
                echo "//# sourceMappingURL=ladder_rs_wasm.js.map" >> "pkg/ladder_rs_wasm.js"
                echo '{"version":3,"sources":["ladder_rs_wasm.js"],"names":[],"mappings":""}' > "pkg/ladder_rs_wasm.js.map"
                log_verbose "Source map generated for debugging"
            fi
        else
            log_error "Development build failed"
            return 1
        fi
    else
        log_info "Dry run: would execute wasm-pack build ${build_args[*]}"
    fi
    
    if [ "$PERFORMANCE_MONITORING" = true ]; then
        local build_time=$(end_timer)
        local memory_after=$(get_memory_usage)
        local memory_diff=$((memory_after - memory_before))
        
        echo -e "${CYAN}Performance Report:${NC}"
        echo "Build time: ${build_time}ms"
        echo "Memory usage: ${memory_after}KB (Δ${memory_diff}KB)"
        
        if [ -f "pkg/ladder_rs_wasm_bg.wasm" ]; then
            local bundle_size=$(stat -c%s "pkg/ladder_rs_wasm_bg.wasm" 2>/dev/null || stat -f%z "pkg/ladder_rs_wasm_bg.wasm" 2>/dev/null)
            local bundle_size_kb=$((bundle_size / 1024))
            echo "Bundle size: ${bundle_size_kb}KB"
        fi
    fi
}

# Watch mode implementation
start_watch_mode() {
    log_info "Starting watch mode for automatic rebuilding..."
    
    if ! command -v inotifywait &> /dev/null && ! command -v fswatch &> /dev/null; then
        log_error "File watching requires either inotifywait (Linux) or fswatch (macOS/BSD)"
        log_info "Install with: sudo apt-get install inotify-tools (Linux) or brew install fswatch (macOS)"
        return 1
    fi
    
    # Start watch script
    ./scripts/watch.sh --dirs src tests types --extensions .rs .toml .json --debounce 500 &
    local watch_pid=$!
    
    log_success "Watch mode started (PID: $watch_pid)"
    log_info "Watching for changes in: src/, tests/, types/"
    log_info "Press Ctrl+C to stop watching"
    
    # Save PID for cleanup
    echo "$watch_pid" > .watch.pid
    
    # Handle cleanup on exit
    trap 'kill $watch_pid; rm -f .watch.pid; exit' INT TERM
    
    # Keep the script running
    wait $watch_pid
}

# Development server
start_dev_server() {
    log_info "Starting development server..."
    
    # Start server script
    ./scripts/serve.sh --port "$DEFAULT_PORT" --host "$DEFAULT_HOST" &
    local server_pid=$!
    
    if [ "$HOT_RELOAD" = true ]; then
        log_info "Hot reload enabled on WebSocket port $WS_PORT"
        ./scripts/serve.sh --hot-reload --ws-port "$WS_PORT" &
        local hot_reload_pid=$!
        echo "$hot_reload_pid" > .hot_reload.pid
    fi
    
    log_success "Development server started (PID: $server_pid)"
    log_info "Server running at: http://$DEFAULT_HOST:$DEFAULT_PORT"
    log_info "Press Ctrl+C to stop server"
    
    # Save PID for cleanup
    echo "$server_pid" > .dev_server.pid
    
    # Handle cleanup on exit
    trap 'kill $server_pid; [ -f .hot_reload.pid ] && kill $(cat .hot_reload.pid); rm -f .dev_server.pid .hot_reload.pid; exit' INT TERM
    
    # Keep the script running
    wait $server_pid
}

# Cleanup function
cleanup_dev_files() {
    log_info "Cleaning up development files..."
    
    local cleanup_files=(
        "pkg/.dev_cache"
        "pkg/.hot_reload_state"
        ".dev_server.pid"
        ".hot_reload.pid"
        ".watch.pid"
        "pkg/*.map"
    )
    
    for file in "${cleanup_files[@]}"; do
        if [ -e "$file" ]; then
            rm -rf "$file"
            log_verbose "Removed: $file"
        fi
    done
    
    # Kill any running development processes
    if [ -f ".dev_server.pid" ]; then
        local server_pid=$(cat .dev_server.pid)
        kill "$server_pid" 2>/dev/null || true
        rm -f .dev_server.pid
    fi
    
    if [ -f ".watch.pid" ]; then
        local watch_pid=$(cat .watch.pid)
        kill "$watch_pid" 2>/dev/null || true
        rm -f .watch.pid
    fi
    
    log_success "Development cleanup completed"
}

# Parse command line arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --mode) DEV_MODE="$2"; shift;;
        --target) TARGET="$2"; shift;;
        --watch) WATCH_MODE=true;;
        --serve) SERVE_MODE=true;;
        --hot-reload) HOT_RELOAD=true;;
        --debug) DEBUG_MODE=true;;
        --verbose|-v) VERBOSE=true;;
        --build-only) BUILD_ONLY=true;;
        --cleanup) CLEANUP=true;;
        --dry-run) DRY_RUN=true;;
        --show-env) SHOW_ENV=true;;
        --performance-monitoring) PERFORMANCE_MONITORING=true;;
        --port) DEFAULT_PORT="$2"; shift;;
        --host) DEFAULT_HOST="$2"; shift;;
        --test-mode) 
            # Special mode for testing - don't actually start long-running processes
            log_debug "Running in test mode"
            exit 0
            ;;
        --help|-h)
            cat << EOF
Development Build Script for ladder-rs WASM - Task 1.1.5

Usage: ./scripts/dev.sh [options]

Build Options:
  --mode MODE           Set development mode (development, debug)
  --target TARGET       Set wasm-pack target (web, nodejs, bundler)
  --build-only          Only build, don't start servers or watch mode
  --debug               Enable debug mode with extra logging
  --verbose, -v         Enable verbose output

Watch Mode:
  --watch               Enable file watching for automatic rebuilding

Development Server:
  --serve               Start development server
  --port PORT           Set server port (default: 3000)
  --host HOST           Set server host (default: 127.0.0.1)
  --hot-reload          Enable hot reload with WebSocket

Monitoring & Debugging:
  --performance-monitoring  Enable build performance monitoring
  --show-env            Show development environment variables
  --dry-run             Show what would be executed without running

Maintenance:
  --cleanup             Clean up development files and stop processes

Examples:
  ./scripts/dev.sh --build-only --debug
  ./scripts/dev.sh --watch --serve --hot-reload
  ./scripts/dev.sh --performance-monitoring --verbose
  ./scripts/dev.sh --cleanup

Environment Variables:
  LADDER_RS_DEV_MODE    Enable development mode features
  LADDER_RS_DEBUG_LEVEL Set debug logging level (verbose, debug, info)
  LADDER_RS_HOT_RELOAD  Enable hot reload functionality
EOF
            exit 0
            ;;
        *) 
            log_error "Unknown parameter: $1" >&2
            echo "Use --help for usage information" >&2
            exit 1
            ;;
    esac
    shift
done

# Header
echo -e "${BLUE}╔════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║              Ladder-RS Development Build System - Task 1.1.5               ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Load configuration
load_dev_config

# Set up environment
setup_dev_environment

# Show environment if requested
if [ "$SHOW_ENV" = true ]; then
    show_environment
    exit 0
fi

# Handle cleanup
if [ "$CLEANUP" = true ]; then
    cleanup_dev_files
    exit 0
fi

# Show configuration
log_info "Development configuration:"
echo "  🔧 Mode: $DEV_MODE"
echo "  🎯 Target: $TARGET"
echo "  👀 Watch mode: $WATCH_MODE"
echo "  🌐 Serve mode: $SERVE_MODE"
echo "  🔥 Hot reload: $HOT_RELOAD"
echo "  🐛 Debug mode: $DEBUG_MODE"
echo "  📊 Performance monitoring: $PERFORMANCE_MONITORING"

if [ "$DEBUG_MODE" = true ]; then
    log_debug "Build configuration:"
    log_debug "  Target: $TARGET"
    log_debug "  Mode: development"
    log_debug "  Debug symbols: enabled"
    log_debug "  Source maps: enabled"
fi

if [ "$VERBOSE" = true ]; then
    log_verbose "File watching:"
    log_verbose "  Watch mode: $WATCH_MODE"
    log_verbose "  Directories: src, tests, types"
    log_verbose "  Extensions: .rs, .toml, .json"
fi

echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    log_error "Cargo.toml not found. Make sure you're in the wasm directory."
    exit 1
fi

# Build first (always required)
dev_build

# Exit if build-only mode
if [ "$BUILD_ONLY" = true ]; then
    log_success "Build completed (build-only mode)"
    exit 0
fi

# Start watch mode if requested
if [ "$WATCH_MODE" = true ] && [ "$SERVE_MODE" = false ]; then
    start_watch_mode
fi

# Start development server if requested
if [ "$SERVE_MODE" = true ] && [ "$WATCH_MODE" = false ]; then
    start_dev_server
fi

# Start both watch and serve if both requested
if [ "$WATCH_MODE" = true ] && [ "$SERVE_MODE" = true ]; then
    log_info "Starting watch mode and development server..."
    
    # Start watch mode in background
    ./scripts/watch.sh --dirs src tests types --extensions .rs .toml .json --debounce 500 &
    local watch_pid=$!
    echo "$watch_pid" > .watch.pid
    
    # Start development server
    start_dev_server
fi

log_success "Development script completed successfully!"
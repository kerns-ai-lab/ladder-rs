#!/bin/bash
# Task 1.1.5: File Watching Script for Development
#
# This script provides intelligent file watching with debouncing,
# file type filtering, and automatic rebuild triggering.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default configuration
WATCH_DIRS=("src")
WATCH_EXTENSIONS=("rs" "toml")
DEBOUNCE_MS=500
RECURSIVE=true
VERBOSE=false
TEST_MODE=false

# Build configuration
BUILD_TARGET="web"
BUILD_MODE="dev"

# Helper functions
log_info() {
    echo -e "${BLUE}[WATCH]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[WATCH]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WATCH]${NC} $1"
}

log_error() {
    echo -e "${RED}[WATCH]${NC} $1"
}

log_verbose() {
    if [ "$VERBOSE" = true ]; then
        echo -e "${CYAN}[WATCH-VERBOSE]${NC} $1"
    fi
}

# Check for file watching tools
check_watch_tools() {
    if command -v inotifywait &> /dev/null; then
        WATCH_TOOL="inotifywait"
        log_verbose "Using inotifywait for file watching"
    elif command -v fswatch &> /dev/null; then
        WATCH_TOOL="fswatch"
        log_verbose "Using fswatch for file watching"
    else
        log_error "No file watching tool found!"
        log_error "Please install either:"
        log_error "  - inotify-tools (Linux): sudo apt-get install inotify-tools"
        log_error "  - fswatch (macOS/BSD): brew install fswatch"
        exit 1
    fi
}

# Build extension filter for watch tools
build_extension_filter() {
    local filter=""
    
    if [ "$WATCH_TOOL" = "inotifywait" ]; then
        # Build regex pattern for inotifywait
        filter=".*\.("
        for i in "${!WATCH_EXTENSIONS[@]}"; do
            if [ $i -gt 0 ]; then
                filter="${filter}|"
            fi
            filter="${filter}${WATCH_EXTENSIONS[$i]}"
        done
        filter="${filter})$"
    elif [ "$WATCH_TOOL" = "fswatch" ]; then
        # For fswatch, we'll build the filters directly in start_fswatch
        # This function returns empty for fswatch since array-based filtering is used
        filter=""
    fi
    
    echo "$filter"
}

# Debounce mechanism
debounce_changes() {
    local last_change_time=0
    local debounce_seconds=$((DEBOUNCE_MS / 1000))
    
    while read -r line; do
        local current_time=$(date +%s)
        local time_diff=$((current_time - last_change_time))
        
        if [ "$time_diff" -ge "$debounce_seconds" ]; then
            log_verbose "File change detected: $line"
            trigger_rebuild "$line"
            last_change_time=$current_time
        else
            log_verbose "File change debounced: $line (${time_diff}s < ${debounce_seconds}s)"
        fi
    done
}

# Trigger rebuild on file changes
trigger_rebuild() {
    local changed_file="$1"
    
    log_info "File changed: $changed_file"
    log_info "Triggering rebuild..."
    
    if [ "$TEST_MODE" = true ]; then
        log_info "Test mode: would rebuild for $changed_file"
        return 0
    fi
    
    # Run the build
    local start_time=$(date +%s%N)
    
    if ../build.sh --target "$BUILD_TARGET" --$BUILD_MODE --no-size-check; then
        local end_time=$(date +%s%N)
        local build_time=$(( (end_time - start_time) / 1000000 )) # Convert to milliseconds
        
        log_success "Rebuild completed in ${build_time}ms"
        
        # Notify any connected development servers about the change
        notify_dev_server "$changed_file"
        
    else
        log_error "Rebuild failed for change in: $changed_file"
    fi
}

# Notify development server about changes (for hot reload)
notify_dev_server() {
    local changed_file="$1"
    
    # Check if hot reload WebSocket server is running
    if [ -f "../.hot_reload.pid" ]; then
        local ws_port=3001
        
        # Send WebSocket message about file change
        # This is a simplified version - real implementation would use proper WebSocket client
        log_verbose "Notifying hot reload server about change: $changed_file"
        
        # You could use tools like wscat, websocat, or a simple curl for WebSocket communication
        # For now, just create a signal file that the server can detect
        mkdir -p "../pkg" && echo "$changed_file" > "../pkg/.hot_reload_trigger"
    fi
}

# Start watching with inotifywait (Linux)
start_inotifywait() {
    local extension_filter=$(build_extension_filter)
    local watch_args=()
    
    # Build watch arguments
    watch_args+=("-m")  # Monitor continuously
    watch_args+=("-e" "modify,create,delete,move")
    watch_args+=("--format" "%w%f %e")
    
    if [ "$RECURSIVE" = true ]; then
        watch_args+=("-r")
    fi
    
    # Add directories to watch
    for dir in "${WATCH_DIRS[@]}"; do
        if [ -d "$dir" ]; then
            watch_args+=("$dir")
        else
            log_warning "Directory not found: $dir"
        fi
    done
    
    log_info "Starting inotifywait with pattern: $extension_filter"
    log_verbose "Command: inotifywait ${watch_args[*]}"
    
    # Start watching and filter by extensions
    inotifywait "${watch_args[@]}" | while read -r line; do
        local file_path=$(echo "$line" | cut -d' ' -f1)
        local event=$(echo "$line" | cut -d' ' -f2)
        
        # Check if file matches extension filter
        if [[ "$file_path" =~ ${extension_filter} ]]; then
            echo "$file_path ($event)"
        fi
    done | debounce_changes
}

# Start watching with fswatch (macOS/BSD)
start_fswatch() {
    local extension_filter=$(build_extension_filter)
    local watch_args=()
    
    # Build watch arguments
    watch_args+=("--one-per-batch")
    watch_args+=("--latency" "0.5")
    
    if [ "$RECURSIVE" = true ]; then
        watch_args+=("--recursive")
    fi
    
    # Add extension filters
    for ext in "${WATCH_EXTENSIONS[@]}"; do
        watch_args+=("--include=.*\\.${ext}$")
    done
    
    # Add directories to watch
    for dir in "${WATCH_DIRS[@]}"; do
        if [ -d "$dir" ]; then
            watch_args+=("$dir")
        else
            log_warning "Directory not found: $dir"
        fi
    done
    
    log_info "Starting fswatch with extensions: ${WATCH_EXTENSIONS[*]}"
    log_verbose "Command: fswatch ${watch_args[*]}"
    
    # Start watching
    fswatch "${watch_args[@]}" | debounce_changes
}

# Parse command line arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --dirs)
            shift
            WATCH_DIRS=()
            while [[ "$#" -gt 0 ]] && [[ "$1" != --* ]]; do
                WATCH_DIRS+=("$1")
                shift
            done
            continue
            ;;
        --extensions)
            shift
            WATCH_EXTENSIONS=()
            while [[ "$#" -gt 0 ]] && [[ "$1" != --* ]]; do
                # Remove leading dot if present
                ext="${1#.}"
                WATCH_EXTENSIONS+=("$ext")
                shift
            done
            continue
            ;;
        --debounce) DEBOUNCE_MS="$2"; shift;;
        --target) BUILD_TARGET="$2"; shift;;
        --mode) BUILD_MODE="$2"; shift;;
        --no-recursive) RECURSIVE=false;;
        --verbose|-v) VERBOSE=true;;
        --test-mode) TEST_MODE=true;;
        --help|-h)
            cat << EOF
File Watching Script for Development - Task 1.1.5

This script provides intelligent file watching with automatic rebuild
triggering, debouncing, and file type filtering.

Usage: ./scripts/watch.sh [options]

Watch Configuration:
  --dirs DIR1 DIR2 ...      Directories to watch (default: src)
  --extensions EXT1 EXT2... File extensions to watch (default: rs toml)
  --debounce MS             Debounce time in milliseconds (default: 500)
  --no-recursive            Don't watch subdirectories recursively

Build Configuration:
  --target TARGET           wasm-pack target for rebuilds (default: web)
  --mode MODE              Build mode: dev, release (default: dev)

Output Options:
  --verbose, -v             Enable verbose output
  --test-mode              Run in test mode (don't actually rebuild)

Examples:
  ./scripts/watch.sh --dirs src tests --extensions rs toml json
  ./scripts/watch.sh --debounce 1000 --verbose
  ./scripts/watch.sh --target nodejs --mode release

Requirements:
  Linux: inotify-tools (sudo apt-get install inotify-tools)
  macOS: fswatch (brew install fswatch)
EOF
            exit 0
            ;;
        *) log_error "Unknown parameter: $1"; echo "Use --help for usage information"; exit 1;;
    esac
    shift
done

# Header
echo -e "${BLUE}╔════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                     File Watching System - Task 1.1.5                     ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check for required tools
check_watch_tools

# Show configuration
log_info "File watching configuration:"
echo "  📁 Directories: ${WATCH_DIRS[*]}"
echo "  📄 Extensions: ${WATCH_EXTENSIONS[*]}"
echo "  ⏱️  Debounce: ${DEBOUNCE_MS}ms"
echo "  🔄 Recursive: $RECURSIVE"
echo "  🎯 Build target: $BUILD_TARGET"
echo "  🏗️  Build mode: $BUILD_MODE"
echo "  🔧 Watch tool: $WATCH_TOOL"
echo ""

# Validate directories
valid_dirs=()
for dir in "${WATCH_DIRS[@]}"; do
    if [ -d "$dir" ]; then
        valid_dirs+=("$dir")
    else
        log_warning "Directory not found, skipping: $dir"
    fi
done

if [ ${#valid_dirs[@]} -eq 0 ]; then
    log_error "No valid directories to watch!"
    exit 1
fi

WATCH_DIRS=("${valid_dirs[@]}")

# Exit early in test mode
if [ "$TEST_MODE" = true ]; then
    log_info "Test mode: watch configuration validated"
    exit 0
fi

# Set up signal handling for graceful shutdown
cleanup() {
    log_info "Stopping file watcher..."
    exit 0
}

trap cleanup INT TERM

# Start the appropriate watcher
log_success "Starting file watcher... (Press Ctrl+C to stop)"
log_info "Watching for changes in: ${WATCH_DIRS[*]}"

if [ "$WATCH_TOOL" = "inotifywait" ]; then
    start_inotifywait
elif [ "$WATCH_TOOL" = "fswatch" ]; then
    start_fswatch
else
    log_error "No supported watch tool available"
    exit 1
fi
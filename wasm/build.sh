#!/bin/bash
# Build script for ladder-rs WASM package

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
BUILD_MODE="dev"
TARGET="web"
OUTPUT_DIR="pkg"

# Parse command line arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --release) BUILD_MODE="release";;
        --dev) BUILD_MODE="dev";;
        --profiling) BUILD_MODE="profiling";;
        --target) TARGET="$2"; shift;;
        --out-dir) OUTPUT_DIR="$2"; shift;;
        --help) 
            echo "Usage: ./build.sh [options]"
            echo "Options:"
            echo "  --release    Build in release mode (optimized)"
            echo "  --dev        Build in development mode (default)"
            echo "  --profiling  Build with profiling optimizations"
            echo "  --target     Set wasm-pack target (web, nodejs, bundler, no-modules)"
            echo "  --out-dir    Set output directory (default: pkg)"
            echo "  --help       Show this help message"
            exit 0
            ;;
        *) echo "Unknown parameter: $1"; exit 1;;
    esac
    shift
done

echo -e "${BLUE}Building ladder-rs WASM package...${NC}"
echo -e "Mode: ${GREEN}$BUILD_MODE${NC}"
echo -e "Target: ${GREEN}$TARGET${NC}"
echo -e "Output: ${GREEN}$OUTPUT_DIR${NC}"

# Clean previous builds
if [ -d "$OUTPUT_DIR" ]; then
    echo -e "${BLUE}Cleaning previous build...${NC}"
    rm -rf "$OUTPUT_DIR"
fi

# Build the package
echo -e "${BLUE}Running wasm-pack build...${NC}"
if [ "$BUILD_MODE" = "release" ]; then
    wasm-pack build --target "$TARGET" --out-dir "$OUTPUT_DIR" --release
elif [ "$BUILD_MODE" = "profiling" ]; then
    wasm-pack build --target "$TARGET" --out-dir "$OUTPUT_DIR" --profiling
else
    wasm-pack build --target "$TARGET" --out-dir "$OUTPUT_DIR" --dev
fi

# Post-build optimizations for release mode
if [ "$BUILD_MODE" = "release" ]; then
    echo -e "${BLUE}Running additional optimizations...${NC}"
    
    # Check if wasm-opt is available
    if command -v wasm-opt &> /dev/null; then
        wasm-opt -Oz \
            --enable-simd \
            --enable-threads \
            "$OUTPUT_DIR/ladder_rs_wasm_bg.wasm" \
            -o "$OUTPUT_DIR/ladder_rs_wasm_bg.wasm"
        echo -e "${GREEN}WASM optimization complete${NC}"
    else
        echo -e "${RED}wasm-opt not found. Skipping additional optimizations.${NC}"
        echo "Install with: npm install -g wasm-opt"
    fi
fi

# Generate size report
echo -e "${BLUE}Build complete! Size report:${NC}"
ls -lh "$OUTPUT_DIR"/*.wasm | awk '{print "WASM size: " $5}'

# Check TypeScript definitions
if [ -f "$OUTPUT_DIR/ladder_rs_wasm.d.ts" ]; then
    echo -e "${GREEN}TypeScript definitions generated successfully${NC}"
else
    echo -e "${RED}Warning: TypeScript definitions not found${NC}"
fi

echo -e "${GREEN}Build completed successfully!${NC}"
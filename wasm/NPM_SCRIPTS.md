# NPM Scripts Documentation - Task 1.1.6

This document provides comprehensive documentation for all npm scripts available in the ladder-rs-wasm package.

## Quick Start

```bash
# Install dependencies
npm install

# Start development mode with watch and hot reload
npm run dev:all

# Build for production
npm run build:release

# Run all tests
npm run test:all
```

## Build Scripts

### Basic Build Commands

- `npm run build` - Build for web target (release mode)
- `npm run build:dev` - Build for web target (development mode)
- `npm run build:release` - Build for web target (release mode, explicit)
- `npm run build:all` - Build for all targets (web, node, bundler)
- `npm run build:all-parallel` - Build all targets in parallel for faster builds

### Target-Specific Builds

- `npm run build:node` - Build specifically for Node.js
- `npm run build:bundler` - Build for bundlers (webpack, rollup, etc.)
- `npm run build:size-check` - Build with verbose output including size information

## Development Scripts

### Core Development Commands

- `npm run dev` - Start development build with debug output
- `npm run dev:watch` - Watch files and rebuild automatically
- `npm run dev:serve` - Start development server with hot reload
- `npm run dev:all` - **Recommended**: Watch + serve + hot reload combined

### Specialized Development Commands

- `npm run dev:watch:build` - Watch files and rebuild (build only, no server)
- `npm run dev:watch:files` - Watch specific file types for changes
- `npm run dev:hot-reload` - Start hot reload server only
- `npm run dev:debug` - Run with debug mode and show environment
- `npm run dev:perf` - Build with performance monitoring enabled

### Convenience Aliases

- `npm run watch` - Alias for `dev:watch`
- `npm run serve` - Alias for `dev:serve`

## Testing Scripts

- `npm test` - Run WASM tests in browsers (Firefox & Chrome)
- `npm run test:node` - Run tests in Node.js environment
- `npm run test:structure` - Validate package structure
- `npm run test:package-json` - Test package.json configuration
- `npm run test:dev-scripts` - Test development scripts integration
- `npm run test:all` - Run all test suites

## Code Quality Scripts

### Formatting and Linting

- `npm run fmt` - Format code using rustfmt
- `npm run fmt:check` - Check code formatting without changes
- `npm run lint` - Run clippy linter with strict warnings
- `npm run check` - Type check for wasm32 target

### Validation Commands

- `npm run check:all` - Run all checks (format, lint, type check)
- `npm run validate` - Validate package structure and configuration
- `npm run verify` - Complete verification (checks + tests + size)

## Utility Scripts

### Cleaning

- `npm run clean` - Remove all build artifacts
- `npm run clean:dev` - Clean development files (PIDs, caches)

### Reporting

- `npm run size-report` - Generate bundle size report for all targets

## Lifecycle Scripts

These run automatically at specific times:

- `prepublishOnly` - Runs before publishing (clean + verify + build)
- `postbuild` - Runs after build (shows generated files)
- `postinstall` - Runs after package installation (shows success message)
- `pretest` - Runs before tests (builds development version)

## Development Workflows

### Basic Development

```bash
# Start development with file watching
npm run dev:watch

# In another terminal, start the dev server
npm run dev:serve
```

### Full Development Setup (Recommended)

```bash
# Start everything in one command
npm run dev:all
```

This starts:
- File watching with automatic rebuilds
- Development server on port 3000
- Hot reload for instant browser updates
- Debug logging for troubleshooting

### Performance Testing

```bash
# Build with performance monitoring
npm run dev:perf

# Check bundle sizes
npm run size-report
```

### Publishing Workflow

```bash
# This runs automatically on `npm publish`
# But you can test it manually:
npm run prepublishOnly
```

This will:
1. Clean all artifacts
2. Run all verification checks
3. Build all targets
4. Prepare for publishing

## Configuration Integration

The npm scripts integrate with:

1. **build.sh** - Core build script for wasm-pack
2. **scripts/dev.sh** - Enhanced development build script
3. **scripts/watch.sh** - File watching system
4. **scripts/serve.sh** - Development server with hot reload

All scripts respect the configuration in:
- `dev.config.json` - Development environment settings
- `Cargo.toml` - Rust/WASM build configuration

## Troubleshooting

### Build Failures

```bash
# Clean and rebuild
npm run clean
npm run build:dev
```

### Watch Mode Issues

```bash
# Clean development files and restart
npm run clean:dev
npm run dev:watch
```

### Test Failures

```bash
# Run specific test suite
npm run test:structure
npm run test:package-json
```

## Environment Variables

The scripts respect these environment variables:

- `LADDER_RS_DEV_MODE` - Enable development features
- `LADDER_RS_DEBUG_LEVEL` - Set debug verbosity
- `LADDER_RS_HOT_RELOAD` - Enable/disable hot reload

## Script Dependencies

Required tools (automatically checked):
- `wasm-pack` - For building WASM modules
- `cargo` - Rust toolchain
- `node` >= 16.0.0
- `npm` >= 8.0.0

Optional tools for enhanced development:
- `inotifywait` (Linux) or `fswatch` (macOS) - For file watching
- `jq` - For JSON processing in scripts
# Task 1.5: CI/CD Integration

**Status:** 🔴 Not Started  
**Estimated Time:** 2 days  
**Priority:** High  
**Assignee:** TBD  

## Description
Set up automated CI/CD pipeline for building, testing, and deploying WASM modules with proper caching and optimization.

## Acceptance Criteria
- [ ] Automated builds on every commit
- [ ] WASM package publishing to npm registry
- [ ] Performance regression detection
- [ ] Cross-platform build verification
- [ ] Artifact caching for faster builds

## Subtasks

### 1.5.1: GitHub Actions Workflow Setup
**Time Estimate:** 6 hours  
**Status:** 🔴 Not Started

#### Description
Create comprehensive GitHub Actions workflows for WASM module CI/CD.

#### Tasks
- [ ] Create main CI workflow for builds and tests
- [ ] Set up WASM-specific build steps
- [ ] Configure matrix builds for different targets
- [ ] Add caching for Rust and Node.js dependencies

#### Main CI Workflow
```yaml
# .github/workflows/wasm-ci.yml
name: WASM CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  wasm-build:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        targets: wasm32-unknown-unknown
        
    - name: Cache Rust dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target/
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
        
    - name: Install wasm-pack
      run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
      
    - name: Build WASM package
      run: |
        cd wasm
        wasm-pack build --target web --release
        
    - name: Check bundle size
      run: |
        cd wasm/pkg
        ls -lah *.wasm
        # Fail if bundle is too large
        test $(stat -c%s *.wasm) -lt 204800 # 200KB limit
        
    - name: Upload WASM artifacts
      uses: actions/upload-artifact@v3
      with:
        name: wasm-package
        path: wasm/pkg/
```

---

### 1.5.2: Automated Testing Pipeline
**Time Estimate:** 8 hours  
**Status:** 🔴 Not Started

#### Description
Integrate all testing frameworks into automated CI pipeline.

#### Tasks
- [ ] Add WASM unit test execution
- [ ] Set up browser testing in CI
- [ ] Configure performance benchmarking
- [ ] Add test result reporting and artifacts

#### Testing Workflow
```yaml
# .github/workflows/wasm-tests.yml
name: WASM Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup Rust and wasm-pack
      # ... (same as build workflow)
      
    - name: Run WASM unit tests
      run: |
        cd wasm
        wasm-pack test --headless --chrome
        wasm-pack test --headless --firefox
        
    - name: Generate test coverage
      run: |
        cd wasm
        cargo tarpaulin --out xml --output-dir coverage/
        
    - name: Upload coverage to Codecov
      uses: codecov/codecov-action@v3
      with:
        file: wasm/coverage/cobertura.xml
        flags: wasm
        
  browser-tests:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        browser: [chrome, firefox]
        
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '18'
        cache: 'npm'
        cache-dependency-path: tests/browser/package-lock.json
        
    - name: Install browser test dependencies
      run: |
        cd tests/browser
        npm ci
        
    - name: Install browsers
      run: npx playwright install --with-deps ${{ matrix.browser }}
      
    - name: Run browser tests
      run: |
        cd tests/browser
        npm test -- --browser ${{ matrix.browser }}
        
  performance-tests:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Run performance benchmarks
      run: |
        cd wasm
        wasm-pack test --headless --chrome -- --bench
        
    - name: Check performance regression
      run: |
        node scripts/check-performance-regression.js
        
    - name: Upload performance results
      uses: actions/upload-artifact@v3
      with:
        name: performance-results
        path: performance-results.json
```

---

### 1.5.3: Package Publishing Automation
**Time Estimate:** 4 hours  
**Status:** 🔴 Not Started

#### Description
Automate publishing of WASM packages to npm registry with proper versioning.

#### Tasks
- [ ] Set up npm package configuration
- [ ] Create automated versioning workflow
- [ ] Configure npm registry authentication
- [ ] Add release automation scripts

#### Publishing Workflow
```yaml
# .github/workflows/publish.yml
name: Publish WASM Package

on:
  release:
    types: [published]
  workflow_dispatch:
    inputs:
      version:
        description: 'Version to publish'
        required: true
        default: 'patch'

jobs:
  publish:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '18'
        registry-url: 'https://registry.npmjs.org'
        
    - name: Install Rust and wasm-pack
      # ... (same setup as other workflows)
      
    - name: Update version
      if: github.event_name == 'workflow_dispatch'
      run: |
        cd wasm
        npm version ${{ github.event.inputs.version }}
        
    - name: Build optimized WASM package
      run: |
        cd wasm
        wasm-pack build --target web --release --scope ladder-rs
        
    - name: Optimize package
      run: |
        cd wasm/pkg
        # Remove unnecessary files
        rm -f .gitignore README.md
        # Optimize wasm file
        wasm-opt -O4 *.wasm -o optimized.wasm
        mv optimized.wasm ladder_rs_wasm.wasm
        
    - name: Publish to npm
      run: |
        cd wasm/pkg
        npm publish --access public
      env:
        NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
        
    - name: Create GitHub release assets
      run: |
        cd wasm/pkg
        tar -czf ../ladder-rs-wasm-${{ github.ref_name }}.tar.gz .
        
    - name: Upload release assets
      uses: actions/upload-release-asset@v1
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      with:
        upload_url: ${{ github.event.release.upload_url }}
        asset_path: wasm/ladder-rs-wasm-${{ github.ref_name }}.tar.gz
        asset_name: ladder-rs-wasm-${{ github.ref_name }}.tar.gz
        asset_content_type: application/gzip
```

---

### 1.5.4: Build Optimization and Caching
**Time Estimate:** 6 hours  
**Status:** 🔴 Not Started

#### Description
Optimize build times and implement effective caching strategies for CI/CD pipeline.

#### Tasks
- [ ] Configure Rust build caching
- [ ] Set up WASM build artifact caching
- [ ] Optimize Docker images for CI
- [ ] Add incremental build support

#### Optimization Strategies
```yaml
# .github/workflows/optimized-builds.yml
name: Optimized Builds

on:
  push:
    branches: [ main ]

jobs:
  cached-build:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0 # Needed for incremental builds
        
    - name: Cache Rust toolchain
      uses: actions/cache@v3
      with:
        path: |
          ~/.rustup
          ~/.cargo/bin
        key: rust-toolchain-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}
        
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target/
          wasm/target/
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
        restore-keys: |
          ${{ runner.os }}-cargo-
          
    - name: Cache wasm-pack installation
      uses: actions/cache@v3
      with:
        path: ~/.cargo/bin/wasm-pack
        key: wasm-pack-${{ runner.os }}-0.12.1
        
    - name: Install wasm-pack (if not cached)
      run: |
        if [ ! -f ~/.cargo/bin/wasm-pack ]; then
          curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
        fi
        
    - name: Check for changes
      id: changes
      run: |
        if git diff --quiet HEAD^ -- wasm/ src/; then
          echo "wasm_changed=false" >> $GITHUB_OUTPUT
        else
          echo "wasm_changed=true" >> $GITHUB_OUTPUT
        fi
        
    - name: Build WASM (if changed)
      if: steps.changes.outputs.wasm_changed == 'true'
      run: |
        cd wasm
        wasm-pack build --target web --release
        
    - name: Use cached WASM build
      if: steps.changes.outputs.wasm_changed == 'false'
      run: |
        echo "Using cached WASM build"
        
  docker-build:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Set up Docker Buildx
      uses: docker/setup-buildx-action@v2
      
    - name: Build Docker image with cache
      uses: docker/build-push-action@v4
      with:
        context: .
        file: docker/Dockerfile.wasm
        cache-from: type=gha
        cache-to: type=gha,mode=max
        push: false
        tags: ladder-rs-wasm:latest
```

#### Docker Optimization
```dockerfile
# docker/Dockerfile.wasm
FROM rust:1.75-slim as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Add WASM target
RUN rustup target add wasm32-unknown-unknown

WORKDIR /app

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./
COPY wasm/Cargo.toml ./wasm/

# Create dummy source files to cache dependencies
RUN mkdir src wasm/src && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > wasm/src/lib.rs

# Build dependencies (this layer will be cached)
RUN cd wasm && cargo build --release --target wasm32-unknown-unknown

# Copy actual source code
COPY src/ ./src/
COPY wasm/src/ ./wasm/src/

# Build the actual project
RUN cd wasm && wasm-pack build --target web --release

FROM nginx:alpine
COPY --from=builder /app/wasm/pkg /usr/share/nginx/html/pkg
COPY --from=builder /app/examples /usr/share/nginx/html/examples
```

## Dependencies
- Task 1.1-1.4 must be completed
- GitHub repository with Actions enabled
- npm registry access tokens
- Docker registry access (if using containerized builds)

## Deliverables
- [ ] Complete CI/CD workflows in `.github/workflows/`
- [ ] Automated testing pipeline
- [ ] Package publishing automation
- [ ] Build optimization configuration
- [ ] Documentation for CI/CD processes

## Risk Factors
- **Low Risk:** CI build failures due to environment issues
- **Low Risk:** Package publishing credential management
- **Medium Risk:** Build time optimization effectiveness

## Testing Checklist
- [ ] All workflows execute successfully on sample commits
- [ ] Package publishing works for test releases
- [ ] Build caching reduces execution time significantly
- [ ] Cross-platform builds produce consistent results
- [ ] Performance regression detection catches actual regressions
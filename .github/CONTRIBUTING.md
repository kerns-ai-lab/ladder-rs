# Contributing to ladder-rs

Thank you for your interest in contributing to ladder-rs! This document provides guidelines and information for contributors.

## Development Setup

### Prerequisites

- Rust 1.70.0 or later (MSRV)
- Git
- For WASM development: Node.js 18+ and wasm-pack

### Clone and Build

```bash
git clone https://github.com/kerns-ai-lab/ladder-rs.git
cd ladder-rs

# Build the project
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench
```

## Code Quality Standards

### Automated Checks

All contributions go through automated quality checks via GitHub Actions:

#### ✅ **Continuous Integration (CI)**
- **Multi-platform testing**: Ubuntu, Windows, macOS
- **Rust version compatibility**: Stable, Beta, MSRV (1.70.0)
- **Code formatting**: `cargo fmt --check`
- **Linting**: `cargo clippy` with deny warnings
- **Documentation**: `cargo doc` with warnings as errors
- **Examples**: All examples must run successfully

#### 🛡️ **Security Auditing**
- **Vulnerability scanning**: Daily security audits with `cargo-audit`
- **Dependency review**: Automated dependency vulnerability checks
- **Supply chain security**: License compliance and duplicate dependency detection
- **Secrets scanning**: Automated detection of leaked secrets
- **OpenSSF Scorecard**: Security best practices assessment

#### 📊 **Code Coverage**
- **Minimum coverage**: 80% line coverage required
- **Coverage reporting**: Automatic coverage reports on PRs
- **Differential coverage**: PRs cannot decrease coverage by >1%
- **Coverage enforcement**: PRs blocked if coverage requirements not met

#### 🏃 **Performance Monitoring**
- **Benchmark regression detection**: Automated performance testing
- **Memory profiling**: Valgrind-based memory leak detection
- **Cross-platform performance**: Performance validation on all platforms
- **Historical tracking**: Performance metrics tracked over time

### Manual Code Standards

#### Code Style
- Follow standard Rust formatting (`cargo fmt`)
- Use `cargo clippy` and address all warnings
- Write comprehensive documentation for public APIs
- Include examples in documentation where helpful

#### Testing Requirements
- Write unit tests for all new functionality
- Include integration tests for complex features
- Add property-based tests using `proptest` where appropriate
- Ensure benchmarks don't regress performance

#### Documentation
- Document all public APIs with examples
- Update README.md for significant changes
- Add relevant examples to the `examples/` directory
- Keep CHANGELOG.md updated

## Pull Request Process

### Before Submitting

1. **Run local checks**:
   ```bash
   cargo fmt
   cargo clippy
   cargo test
   cargo doc --no-deps
   ```

2. **Check coverage locally**:
   ```bash
   # Install cargo-llvm-cov if not already installed
   cargo install cargo-llvm-cov
   
   # Generate coverage report
   cargo llvm-cov --html
   # Open target/llvm-cov/html/index.html to view
   ```

3. **Run benchmarks**:
   ```bash
   cargo bench
   ```

### PR Guidelines

1. **Create focused PRs**: Keep changes small and focused on a single feature/fix
2. **Write clear descriptions**: Explain what changes you made and why
3. **Reference issues**: Link to relevant issues using `Fixes #123` or `Relates to #123`
4. **Add tests**: Include appropriate tests for your changes
5. **Update documentation**: Keep docs in sync with code changes

### Automated PR Checks

Your PR will automatically trigger:

- ✅ **CI Pipeline**: Full test suite across platforms
- 🛡️ **Security Scans**: Vulnerability and dependency checks  
- 📊 **Coverage Analysis**: Coverage reporting and enforcement
- 🏃 **Performance Tests**: Benchmark regression detection

PRs cannot be merged until all checks pass and coverage requirements are met.

## Release Process

Releases are automated through GitHub Actions:

### Version Bumping
- Follow [Semantic Versioning](https://semver.org/)
- Update version in `Cargo.toml` and `wasm/Cargo.toml`
- Update `CHANGELOG.md` with release notes

### Release Workflow
1. **Tag creation**: Push a git tag like `v1.2.3`
2. **Automated testing**: Full test suite runs on all platforms
3. **Artifact building**: Cross-platform binaries and WASM package
4. **GitHub Release**: Automated release creation with changelog
5. **Package publishing**: Automatic publishing to crates.io and npm
6. **Documentation**: Updated API docs deployed to GitHub Pages

### Manual Release
You can also trigger releases manually via GitHub Actions workflow dispatch.

## Development Workflows

### Adding New Rating Algorithms

1. **Create algorithm module**: Add to `src/` directory
2. **Implement core traits**: `RatingSystem`, `Rating`, `TeamRating`
3. **Add comprehensive tests**: Unit, integration, and property tests
4. **Include benchmarks**: Add performance benchmarks
5. **Write documentation**: API docs with examples
6. **Add examples**: Working example in `examples/` directory

### Working with WASM

```bash
cd wasm

# Build WASM package
./build.sh

# Test WASM package
npm test

# Publish (automated in CI)
npm publish
```

### Updating Dependencies

1. **Security first**: Run `cargo audit` before and after updates
2. **Test thoroughly**: Ensure all tests pass with new dependencies
3. **Check licenses**: Verify license compatibility with `cargo deny check`
4. **Update lockfile**: Commit `Cargo.lock` changes

## Getting Help

- **Issues**: Create GitHub issues for bugs or feature requests
- **Discussions**: Use GitHub Discussions for questions
- **Security**: Report security issues privately via GitHub Security tab

## Recognition

Contributors are recognized in:
- GitHub contributor graphs
- Release notes for significant contributions
- Repository README (for major contributions)

Thank you for helping make ladder-rs better! 🦀
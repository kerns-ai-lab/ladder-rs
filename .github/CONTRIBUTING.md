# Contributing to ladder-rs

Thank you for your interest in contributing to ladder-rs!

## Development Setup

### Prerequisites

- Rust 1.70.0 or later
- Git

### Getting Started

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

## Pull Request Process

### Before Submitting

1. **Format your code**: `cargo fmt`
2. **Check for warnings**: `cargo clippy`
3. **Run tests**: `cargo test`
4. **Build docs**: `cargo doc --no-deps`

### PR Guidelines

1. Keep changes focused on a single feature or fix
2. Write clear commit messages
3. Include tests for new functionality
4. Update documentation as needed

### Automated Checks

All PRs run through CI which checks:
- Code formatting (`cargo fmt`)
- Linting (`cargo clippy`)
- Tests on Ubuntu, Windows, and macOS
- Minimum supported Rust version (1.70.0)
- Code coverage reporting

## Code Style

- Use `cargo fmt` for formatting
- Address all `cargo clippy` warnings
- Write tests for new features
- Document public APIs
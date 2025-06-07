# ladder-rs

A high-performance, extensible matchmaking library in Rust.

## Features

- Modular architecture with trait-based design for extensibility
- Multiple rating systems:
  - TrueSkill (Microsoft's Bayesian skill rating system)
  - Elo (classic chess rating system)
  - Glicko and Glicko-2 (improved rating systems with rating deviation)
- Type-safe and mathematically sound implementations
- Comprehensive test suite with unit, integration, and property-based tests
- Performance benchmarks

## Implementation Plan

This library is being developed in the following phases:

1. **Core Abstractions & Library Foundation** - Define core traits and types
2. **TrueSkill Implementation: Foundational Elements** - Data structures and initial setup
3. **TrueSkill Implementation: Core Algorithm & Message Passing** - Core algorithm implementation
4. **TrueSkill Implementation: Features & API** - API completion
5. **Elo Rating System Implementation** - Complete Elo implementation
6. **Glicko & Glicko-2 Rating Systems Implementation** - Complete Glicko implementations
7. **Finalization, Performance, Testing & Documentation** - Final polish and testing

## Usage

```rust
// Example code will be provided when the library is more complete
```

## License

MIT
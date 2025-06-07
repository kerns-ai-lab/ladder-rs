# ladder-rs

A high-performance, extensible matchmaking library in Rust.

## Features

- Modular architecture with trait-based design for extensibility
- Rating systems implemented from scratch:
  - **Elo**
  - **Glicko** and **Glicko-2**
  - **TrueSkill** (rating updates not yet implemented)
- Type-safe and mathematically sound implementations
- Comprehensive unit and integration tests
- Performance benchmarks (criterion)

## Project Status

The Elo, Glicko, and Glicko-2 algorithms are fully functional with tests. The
TrueSkill module currently provides data structures and parameter validation but
does not yet perform rating updates or match quality calculations.

## Usage

Below is a minimal example using the Elo rating system:

```rust
use ladder_rs::{elo::{EloRating, EloTeamRating, EloSystem}, core::{GameOutcome, RatingSystem}};

fn main() -> ladder_rs::error::Result<()> {
    let system = EloSystem::new();
    let team1 = EloTeamRating::new(EloRating::new(1500.0));
    let team2 = EloTeamRating::new(EloRating::new(1500.0));

    let outcome = GameOutcome::win(0, 2); // team1 wins
    let updated = system.rate(&[team1, team2], &outcome)?;

    println!("New ratings: {:?}", updated);
    Ok(())
}
```

Run tests with `cargo test` and benchmarks with `cargo bench` (benchmarks are
placeholders until the final development phase).

## Implementation Plan

This library is being developed in the following phases:

1. **Core Abstractions & Library Foundation**
2. **TrueSkill Implementation: Foundational Elements**
3. **TrueSkill Implementation: Core Algorithm & Message Passing**
4. **TrueSkill Implementation: Features & API**
5. **Elo Rating System Implementation**
6. **Glicko & Glicko-2 Rating Systems Implementation**
7. **Finalization, Performance, Testing & Documentation**

## License

MIT

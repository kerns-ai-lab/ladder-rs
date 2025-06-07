# Ladder-RS Examples

This directory contains examples demonstrating the usage of the ladder-rs rating system library.

## Running Examples

To run any example, use:

```bash
cargo run --example <example_name>
```

## Available Examples

### Basic Examples

#### 1. `elo_basic.rs` - Basic Elo Rating System
Demonstrates fundamental Elo rating operations with 1v1 matches.

```bash
cargo run --example elo_basic
```

#### 2. `glicko_basic.rs` - Basic Glicko Rating System  
Shows Glicko ratings with uncertainty (Rating Deviation).

```bash
cargo run --example glicko_basic
```

#### 3. `trueskill_basic.rs` - Basic TrueSkill System
Microsoft's Bayesian rating system with Gaussian distributions.

```bash
cargo run --example trueskill_basic
```

#### 4. `rating_comparison.rs` - System Comparison
Side-by-side comparison of Elo, Glicko, and TrueSkill systems.

```bash
cargo run --example rating_comparison
```

## System Overview

| System | Complexity | Uncertainty | Best For |
|--------|------------|-------------|-----------|
| **Elo** | Simple | None | Chess, simple 1v1 games |
| **Glicko** | Moderate | Rating Deviation | Periodic tournaments |
| **TrueSkill** | High | Full Bayesian | Team games, Xbox Live |

## Key Concepts

- **Rating**: The system's estimate of player skill
- **Conservative Rating**: Lower bound estimate for fair matchmaking
- **Match Quality**: Prediction of how competitive a match will be
- **Uncertainty**: How confident the system is about a player's skill

For more detailed examples and advanced usage, see the individual example files.
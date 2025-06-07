# Ladder-RS Examples

This directory contains comprehensive examples demonstrating the usage of the ladder-rs rating system library. The examples cover all major rating systems (Elo, Glicko, Glicko-2, and TrueSkill) and showcase practical applications.

## Running Examples

To run any example, use:

```bash
cargo run --example <example_name>
```

For example:
```bash
cargo run --example elo_basic
```

## Available Examples

### Basic System Examples

#### 1. `elo_basic.rs` - Basic Elo Rating System
- Demonstrates fundamental Elo rating operations
- Shows 1v1 matches with wins, losses, and draws
- Explains rating updates and match quality calculation
- **Key concepts**: K-factor, rating updates, win probability

```bash
cargo run --example elo_basic
```

#### 2. `glicko_basic.rs` - Basic Glicko Rating System  
- Introduces Rating Deviation (RD) concept
- Shows how uncertainty affects rating updates
- Demonstrates time passage effects on ratings
- **Key concepts**: Rating deviation, uncertainty, time decay

```bash
cargo run --example glicko_basic
```

#### 3. `glicko2_advanced.rs` - Advanced Glicko-2 System
- Showcases volatility (σ) in addition to rating and RD
- Custom system parameters for different game types
- Demonstrates performance consistency tracking
- **Key concepts**: Volatility, performance consistency, conservative ratings

```bash
cargo run --example glicko2_advanced
```

#### 4. `trueskill_basic.rs` - Basic TrueSkill System
- Microsoft's Bayesian rating system fundamentals
- Gaussian skill distributions (μ, σ)
- Conservative rating calculations
- **Key concepts**: Bayesian inference, skill uncertainty, conservative estimates

```bash
cargo run --example trueskill_basic
```

#### 5. `trueskill_teams.rs` - TrueSkill Team Concepts
- Explains team-based rating concepts (conceptual)
- Individual player development through 1v1 matches
- Team formation strategies
- **Key concepts**: Team ratings, balanced matchmaking, skill aggregation

```bash
cargo run --example trueskill_teams
```

### Advanced Examples

#### 6. `elo_tournament.rs` - Tournament Simulation
- Complete round-robin tournament using Elo
- Multiple players with different skill levels
- Match outcome simulation based on ratings
- Tournament standings and analysis
- **Use case**: Chess tournaments, competitive gaming

```bash
cargo run --example elo_tournament
```

#### 7. `rating_comparison.rs` - System Comparison
- Side-by-side comparison of all rating systems
- Same match outcomes across different systems
- Analysis of system characteristics and trade-offs
- **Use case**: Choosing the right rating system for your application

```bash
cargo run --example rating_comparison
```

#### 8. `matchmaking_system.rs` - Complete Matchmaking
- Full matchmaking system implementation
- Multiple rating systems working together
- Match quality optimization
- Leaderboard generation across systems
- Performance analysis and insights
- **Use case**: Online gaming, competitive platforms

```bash
cargo run --example matchmaking_system
```

## Rating System Comparison

| System | Complexity | Team Support | Uncertainty | Best For |
|--------|------------|--------------|-------------|-----------|
| **Elo** | Simple | No | None | Chess, simple 1v1 games |
| **Glicko** | Moderate | No | Rating Deviation | Periodic tournaments |
| **Glicko-2** | High | No | RD + Volatility | Competitive gaming |
| **TrueSkill** | Very High | Yes | Full Bayesian | Team games, Xbox Live |

## Key Concepts Explained

### Rating vs Conservative Rating
- **Rating**: The system's best estimate of skill
- **Conservative Rating**: Lower bound estimate accounting for uncertainty
- **Usage**: Use conservative ratings for leaderboards and fair matchmaking

### Match Quality
- Predicts how competitive/entertaining a match will be
- Higher values indicate more balanced matches
- Used by matchmaking systems to create fair games

### Uncertainty Handling
- **Elo**: No explicit uncertainty (treats all ratings as certain)
- **Glicko**: Rating Deviation (RD) represents uncertainty
- **Glicko-2**: RD + Volatility for performance consistency
- **TrueSkill**: Full Gaussian distribution with mean and variance

### System Selection Guide

#### Choose Elo if:
- You need simplicity and speed
- Working with 1v1 games only
- Don't need uncertainty measures
- Following traditional chess/Go rating systems

#### Choose Glicko if:
- You have rating periods (tournaments, seasons)
- Want uncertainty representation
- Need time-based rating decay
- Working with established competitive formats

#### Choose Glicko-2 if:
- Need sophisticated uncertainty modeling
- Players have varying performance consistency
- Want state-of-the-art individual rating system
- Can handle computational complexity

#### Choose TrueSkill if:
- Working with team-based games
- Need multi-player support
- Want Bayesian statistical rigor
- Can implement full factor graph algorithm

## Implementation Notes

### Current Limitations
- TrueSkill implementation currently limited to 1v1 (simplified version)
- Full factor graph implementation planned for future releases
- Match quality calculation not yet implemented for TrueSkill

### Performance Considerations
- Elo: Fastest, O(1) updates
- Glicko: Moderate, O(n) where n = number of opponents
- Glicko-2: Slower due to iterative volatility calculation
- TrueSkill: Most complex, especially for large teams

### Integration Tips
1. Start with Elo for prototyping
2. Add uncertainty with Glicko/Glicko-2 for production
3. Use conservative ratings for public leaderboards
4. Monitor match quality for balanced gameplay
5. Consider hybrid approaches (multiple systems)

## Contributing

When adding new examples:
1. Follow the existing naming pattern
2. Include comprehensive comments explaining concepts
3. Show both successful and edge cases
4. Update this README with the new example
5. Add the example to `Cargo.toml` if needed

## Additional Resources

- [TrueSkill Paper](https://www.microsoft.com/en-us/research/publication/trueskilltm-a-bayesian-skill-rating-system/)
- [Glicko-2 System](http://www.glicko.net/glicko/glicko2.pdf)
- [Elo Rating System](https://en.wikipedia.org/wiki/Elo_rating_system)
- [Rating System Comparison](https://www.moserware.com/2010/03/computing-your-skill.html)
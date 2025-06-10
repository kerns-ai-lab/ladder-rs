# Unified Rating System Interface

This document describes the unified rating system interface implemented for Task 1.3.1.

## Overview

The unified rating system provides a consistent JavaScript API across all rating algorithms (Elo, Glicko, TrueSkill). Currently, only Elo is fully implemented, with Glicko and TrueSkill support planned for future iterations.

## API Design

### Core Types

1. **RatingSystemType**: Enum identifying the rating algorithm
   - `Elo`
   - `Glicko` (planned)
   - `TrueSkill` (planned)

2. **PlayerInfo**: Player rating information
   - `id`: Player identifier
   - `rating`: Current rating value
   - `uncertainty`: Rating uncertainty/deviation
   - `conservative_rating`: Conservative estimate (TrueSkill only)
   - `matches_played`: Number of matches played

3. **MatchResult**: Result of a processed match
   - `winner_team`: Which team won (1 or 2)
   - `updated_ratings`: Array of updated PlayerInfo
   - `match_quality`: Match quality/balance (0-1)

### UnifiedRatingSystem Class

Main class providing unified access to all rating systems.

#### Constructor
```javascript
const system = new UnifiedRatingSystem({
  system: "elo",    // Rating system type
  k_factor: 32      // System-specific parameters
});
```

#### Player Management
- `create_player(player_id)`: Create a new player
- `create_players([player_ids])`: Batch create players
- `get_player(player_id)`: Get player information

#### Match Processing
- `process_match(team1_players, team2_players, winner_team)`: Process a single match
- `process_matches([matches])`: Batch process matches

#### Analytics
- `calculate_match_quality(team1, team2)`: Calculate match balance
- `predict_win_probability(team1, team2)`: Predict team 1 win probability
- `get_leaderboard(limit?)`: Get sorted player rankings

#### Persistence
- `serialize()`: Export system state to JSON
- `deserialize(data)`: Restore system from JSON

## Usage Examples

### Basic 1v1 Match
```javascript
const system = new UnifiedRatingSystem({ system: "elo" });

// Create players
system.create_player("alice");
system.create_player("bob");

// Process a match (alice wins)
const result = system.process_match(
  ["alice"],  // Team 1
  ["bob"],    // Team 2
  1           // Team 1 wins
);

console.log(result.updated_ratings);
```

### Team Match
```javascript
// 2v2 match
const result = system.process_match(
  ["alice", "bob"],      // Team 1
  ["charlie", "david"],  // Team 2
  2                      // Team 2 wins
);
```

### Match Quality Assessment
```javascript
// Check if a match would be balanced
const quality = system.calculate_match_quality(["alice"], ["bob"]);
if (quality > 0.8) {
  console.log("This would be a well-balanced match!");
}
```

### Leaderboard
```javascript
// Get top 10 players
const top10 = system.get_leaderboard(10);
top10.forEach((player, rank) => {
  console.log(`${rank + 1}. ${player.id}: ${player.rating}`);
});
```

### State Persistence
```javascript
// Save state
const savedState = system.serialize();
localStorage.setItem('ratings', JSON.stringify(savedState));

// Later: restore state
const savedData = JSON.parse(localStorage.getItem('ratings'));
const system = UnifiedRatingSystem.deserialize(savedData);
```

## Implementation Status

### Completed
- ✅ Unified interface design
- ✅ Elo rating system integration
- ✅ Player management
- ✅ Match processing (1v1 and team)
- ✅ Match quality calculation
- ✅ Leaderboard generation
- ✅ State serialization/deserialization
- ✅ Comprehensive error handling
- ✅ Batch operations

### Pending
- ⏳ Glicko rating system integration
- ⏳ TrueSkill rating system integration
- ⏳ Rating period decay (Glicko)
- ⏳ Draw support
- ⏳ Multi-team matches (>2 teams)

## Technical Details

### Bundle Size
The unified interface adds minimal overhead to the WASM bundle:
- Core interface: ~5KB (uncompressed)
- With Elo only: ~15KB total (gzipped)

### Error Handling
All methods use the comprehensive error handling framework from Task 1.2.5:
- Type-safe error codes
- Detailed error messages
- JavaScript Error object compatibility

### Performance
- O(1) player lookup
- O(n) leaderboard generation (where n = number of players)
- Efficient batch operations

## Future Enhancements

1. **Full Algorithm Support**: Implement Glicko and TrueSkill adapters
2. **Advanced Features**: 
   - Rating confidence intervals
   - Skill progression tracking
   - Tournament support
3. **Optimization**: 
   - Lazy leaderboard updates
   - Indexed player lookups
   - Streaming match processing

## Testing

Comprehensive tests are provided in `tests/unified_rating_system_tests.rs`, covering:
- System creation and configuration
- Player management
- Match processing (1v1 and team)
- Analytics functions
- Serialization/deserialization
- Error cases
- Batch operations

Run tests with:
```bash
cd wasm && wasm-pack test --node
```
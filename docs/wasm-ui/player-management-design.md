# Player Management System Design (Task 1.3.2)

## Overview
The Player Management System provides comprehensive player tracking, profile management, match history, and statistics calculation for the ladder-rs WASM module.

## Key Features

### 1. Player Registration & Profiles
- Unique player IDs
- Optional name and email fields
- Timestamps for creation and updates
- Active/inactive status for soft deletion

### 2. Match History Tracking
- Unique match IDs using UUID v4
- Support for team matches (multiple players per team)
- Outcome tracking (win/loss/draw)
- Timestamps for all matches
- Optional notes for matches

### 3. Player Statistics
- Total matches played
- Wins, losses, and draws
- Win rate calculation
- Current streak tracking (positive for wins, negative for losses)
- Longest win/loss streak tracking

### 4. Head-to-Head Records
- Direct comparison between two players
- Tracks total matches between specific players
- Individual win counts for each player
- Draw count

### 5. Player Management Features
- Search by name or ID (case-insensitive)
- Bulk import/export functionality (JSON format)
- Player aliases support
- Merge duplicate players
- Active player filtering

## Implementation Details

### Data Structures

```rust
PlayerProfile {
    id: String,
    name: Option<String>,
    email: Option<String>,
    created_at: f64,
    updated_at: f64,
    is_active: bool,
}

MatchRecord {
    id: String,
    team1_players: Vec<String>,
    team2_players: Vec<String>,
    outcome: i32, // 0 = draw, 1 = team1 wins, 2 = team2 wins
    timestamp: f64,
    notes: Option<String>,
}

PlayerStats {
    player_id: String,
    total_matches: u32,
    wins: u32,
    losses: u32,
    draws: u32,
    win_rate: f64,
    current_streak: i32,
    longest_win_streak: u32,
    longest_loss_streak: u32,
}
```

### Storage
- Players stored in HashMap with ID as key
- Matches stored in separate HashMap
- Alias mapping maintained for quick lookups
- Player data includes profile, match IDs, and aliases

### JavaScript API

```javascript
// Create manager
const manager = new PlayerManager();

// Register players
const player = manager.register_player("alice123", "Alice Smith", "alice@example.com");

// Record match
const matchId = manager.add_match_record(
    ["player1"], // team1
    ["player2"], // team2
    1,           // outcome (team1 wins)
    "Tournament final"
);

// Get statistics
const stats = manager.get_player_stats("player1");
console.log(`Win rate: ${stats.win_rate}`);

// Search players
const results = manager.search_players("Smith");

// Import/Export
const json = manager.export_players(true); // include inactive
const imported = manager.bulk_import_players(jsonData);
```

## Integration with Rating Systems

The Player Management System is designed to work alongside the WasmRatingSystem:

1. Player IDs from PlayerManager can be used with WasmRatingSystem
2. Match results can be tracked in both systems
3. Leaderboards can combine rating data with player profiles
4. Statistics complement rating calculations

## Future Enhancements

1. **Match Metadata**: Additional fields like game duration, tournament info
2. **Player Tags**: Categories and labels for players
3. **Team Management**: Persistent team formations
4. **Achievements**: Milestone tracking based on statistics
5. **ELO History**: Integration to track rating changes over time
6. **Pagination**: More sophisticated pagination for large datasets
7. **Filtering**: Advanced filtering options for match history

## Testing

Comprehensive test coverage includes:
- Player CRUD operations
- Match recording and history
- Statistics calculation accuracy
- Head-to-head records
- Search functionality
- Import/export validation
- Alias management
- Player merging

All tests pass using wasm-bindgen-test framework.
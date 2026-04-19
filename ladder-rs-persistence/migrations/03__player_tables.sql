-- Migration V3: Player tables
-- Creates players, league_players, and player_aliases tables

CREATE TABLE IF NOT EXISTS players (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    nickname TEXT,
    player_type TEXT NOT NULL DEFAULT 'human',
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_players_name ON players(name);

CREATE TABLE IF NOT EXISTS league_players (
    id TEXT NOT NULL PRIMARY KEY,
    league_id TEXT NOT NULL REFERENCES leagues(id) ON DELETE RESTRICT,
    player_id TEXT NOT NULL REFERENCES players(id) ON DELETE RESTRICT,
    is_active INTEGER NOT NULL DEFAULT 1,
    joined_at TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    UNIQUE(league_id, player_id)
);

CREATE TABLE IF NOT EXISTS player_aliases (
    id TEXT NOT NULL PRIMARY KEY,
    primary_player_id TEXT NOT NULL REFERENCES players(id) ON DELETE RESTRICT,
    alias_player_id TEXT NOT NULL REFERENCES players(id) ON DELETE RESTRICT,
    created_by TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    UNIQUE(primary_player_id, alias_player_id)
);

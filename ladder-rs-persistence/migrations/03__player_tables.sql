-- Migration V3: Player tables
-- Creates players, league_players, and player_aliases tables

CREATE TABLE IF NOT EXISTS players (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    nickname TEXT,
    player_type TEXT NOT NULL DEFAULT 'human',
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now')),
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_players_name ON players(name);

CREATE TABLE IF NOT EXISTS league_players (
    id INTEGER PRIMARY KEY,
    league_id INTEGER NOT NULL REFERENCES leagues(id) ON DELETE RESTRICT,
    player_id INTEGER NOT NULL REFERENCES players(id) ON DELETE RESTRICT,
    is_active INTEGER NOT NULL DEFAULT 1,
    joined_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now')),
    UNIQUE(league_id, player_id)
);

CREATE TABLE IF NOT EXISTS player_aliases (
    id INTEGER PRIMARY KEY,
    primary_player_id INTEGER NOT NULL REFERENCES players(id) ON DELETE RESTRICT,
    alias_player_id INTEGER NOT NULL REFERENCES players(id) ON DELETE RESTRICT,
    created_by INTEGER NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now')),
    UNIQUE(primary_player_id, alias_player_id)
);

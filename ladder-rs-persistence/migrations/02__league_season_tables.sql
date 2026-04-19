-- Migration V2: League and Season tables
-- Creates leagues, league_operators, and seasons tables

CREATE TABLE IF NOT EXISTS leagues (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    algorithm TEXT NOT NULL,
    visibility TEXT NOT NULL DEFAULT 'public',
    is_active INTEGER NOT NULL DEFAULT 1,
    is_archived INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_leagues_visibility_archived ON leagues(visibility, is_archived);

CREATE TABLE IF NOT EXISTS league_operators (
    id TEXT NOT NULL PRIMARY KEY,
    league_id TEXT NOT NULL REFERENCES leagues(id) ON DELETE RESTRICT,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(league_id, user_id)
);

CREATE INDEX idx_league_operators_league_user ON league_operators(league_id, user_id);

CREATE TABLE IF NOT EXISTS seasons (
    id TEXT NOT NULL PRIMARY KEY,
    league_id TEXT NOT NULL REFERENCES leagues(id) ON DELETE RESTRICT,
    algorithm TEXT NOT NULL,
    params_json TEXT,
    start_date TEXT NOT NULL,
    end_date TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

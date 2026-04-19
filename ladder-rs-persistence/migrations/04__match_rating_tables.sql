-- Migration V4: Match and Rating tables
-- Creates matches, match_participants, match_audit_log, rating_snapshots, and recalculation_jobs

CREATE TABLE IF NOT EXISTS matches (
    id TEXT NOT NULL PRIMARY KEY,
    season_id TEXT NOT NULL REFERENCES seasons(id) ON DELETE RESTRICT,
    recorded_at TEXT NOT NULL,  -- NO DEFAULT — application must supply for rating ordering
    score_metadata_json TEXT,
    is_corrected INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP)
);

CREATE INDEX idx_matches_season_recorded ON matches(season_id, recorded_at DESC);

CREATE TABLE IF NOT EXISTS match_participants (
    id TEXT NOT NULL PRIMARY KEY,
    match_id TEXT NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    player_id TEXT NOT NULL REFERENCES players(id) ON DELETE RESTRICT,
    placement INTEGER NOT NULL,
    rating_before TEXT,
    rating_after TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_match_participants_match ON match_participants(match_id);
CREATE INDEX idx_match_participants_player_season ON match_participants(player_id);

CREATE TABLE IF NOT EXISTS match_audit_log (
    id TEXT NOT NULL PRIMARY KEY,
    match_id TEXT NOT NULL REFERENCES matches(id) ON DELETE RESTRICT,
    actor_user_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    original_data_json TEXT NOT NULL,
    corrected_data_json TEXT NOT NULL,
    reason TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS rating_snapshots (
    id TEXT NOT NULL PRIMARY KEY,
    season_id TEXT NOT NULL REFERENCES seasons(id) ON DELETE RESTRICT,
    player_id TEXT NOT NULL REFERENCES players(id) ON DELETE RESTRICT,
    match_id TEXT NOT NULL REFERENCES matches(id) ON DELETE RESTRICT,
    conservative_rating REAL NOT NULL,
    rating_json TEXT NOT NULL,
    timestamp TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP)
);

CREATE INDEX idx_rating_snapshots_season_rating ON rating_snapshots(season_id, conservative_rating DESC);
CREATE INDEX idx_rating_snapshots_player_season_time ON rating_snapshots(player_id, season_id, timestamp ASC);

CREATE TABLE IF NOT EXISTS recalculation_jobs (
    id TEXT NOT NULL PRIMARY KEY,
    season_id TEXT NOT NULL REFERENCES seasons(id) ON DELETE RESTRICT,
    job_type TEXT NOT NULL,  -- 'alias_link', 'alias_unlink', 'match_correction'
    status TEXT NOT NULL DEFAULT 'queued',  -- 'queued', 'in_progress', 'completed', 'failed'
    created_at TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    completed_at TEXT
);

CREATE INDEX idx_recalc_jobs_status_created ON recalculation_jobs(status, created_at ASC);

-- Migration V7: Recreate invite_tokens table with updated schema
-- The old invite_tokens table had user_id (FK to users), token, used_at.
-- The new schema uses player_id (plain string), token_hash, created_by (FK to users),
-- claimed_by, and claimed_at.
--
-- Since the semantics changed (user_id → player_id no longer references users),
-- we drop and recreate the table. Data loss is acceptable during early development.

DROP TABLE IF EXISTS invite_tokens;

CREATE TABLE invite_tokens (
    id TEXT NOT NULL PRIMARY KEY,
    player_id TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    created_by TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    claimed_by TEXT,
    claimed_at TEXT,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

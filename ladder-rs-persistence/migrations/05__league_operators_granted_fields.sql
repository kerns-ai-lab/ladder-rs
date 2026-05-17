-- Migration V5: Add granted_by and granted_at columns to league_operators
-- Supports the LeagueOperator model with grant tracking fields.

ALTER TABLE league_operators ADD COLUMN granted_by TEXT;
ALTER TABLE league_operators ADD COLUMN granted_at TEXT;

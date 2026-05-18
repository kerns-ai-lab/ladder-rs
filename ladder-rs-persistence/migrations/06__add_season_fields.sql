-- Migration V5: Add season management fields
-- Adds number (auto-increment per league) and is_open (boolean) to seasons

ALTER TABLE seasons ADD COLUMN number INTEGER NOT NULL DEFAULT 0;
ALTER TABLE seasons ADD COLUMN is_open INTEGER NOT NULL DEFAULT 1;

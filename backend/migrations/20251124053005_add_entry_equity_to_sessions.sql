-- Add migration script here
-- Track equity at the moment a position is opened to allow drift-free MTM updates
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS entry_equity DOUBLE PRECISION;

-- Initialize existing sessions with their current equity as the starting basis
UPDATE sessions SET entry_equity = current_equity WHERE entry_equity IS NULL;

-- Add Merkle hash columns to mission_logs for high-fidelity audit integrity
-- Each log entry now carries a 'hash' of its content + the 'prev_hash' of the preceding entry,
-- forming an immutable chain of custody for mission execution.

ALTER TABLE mission_logs ADD COLUMN hash TEXT;
ALTER TABLE mission_logs ADD COLUMN prev_hash TEXT;

-- Index for optimized Merkle chain reconciliation
CREATE INDEX idx_mission_logs_hashes ON mission_logs (mission_id, timestamp);

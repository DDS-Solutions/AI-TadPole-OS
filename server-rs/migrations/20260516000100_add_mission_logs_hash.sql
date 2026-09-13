-- Add hash and prev_hash for Merkle Audit Trail
ALTER TABLE mission_logs ADD COLUMN hash TEXT;
ALTER TABLE mission_logs ADD COLUMN prev_hash TEXT;

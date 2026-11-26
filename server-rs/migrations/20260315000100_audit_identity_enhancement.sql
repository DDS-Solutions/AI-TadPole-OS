-- Add mission_id and user_id to audit_trail for identity traceability (IBM Agentic Security Research)
ALTER TABLE audit_trail ADD COLUMN mission_id TEXT;
ALTER TABLE audit_trail ADD COLUMN user_id TEXT;

-- Index for mission-specific audit lookups
CREATE INDEX IF NOT EXISTS idx_audit_mission ON audit_trail(mission_id);
CREATE INDEX IF NOT EXISTS idx_audit_user ON audit_trail(user_id);

-- @docs ARCHITECTURE:Persistence
-- Additive ledger; older engines can safely leave this table in place.
CREATE TABLE IF NOT EXISTS delegation_assignments (
    assignment_key TEXT PRIMARY KEY NOT NULL,
    mission_id TEXT NOT NULL,
    source_agent_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('running', 'completed', 'failed')),
    result TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_delegation_assignments_mission ON delegation_assignments(mission_id);
CREATE TRIGGER IF NOT EXISTS delegation_assignments_delete_mission
AFTER DELETE ON mission_history
BEGIN
    DELETE FROM delegation_assignments WHERE mission_id = OLD.id;
END;
CREATE TRIGGER IF NOT EXISTS delegation_assignments_delete_source
AFTER DELETE ON agents
BEGIN
    DELETE FROM delegation_assignments WHERE source_agent_id = OLD.id;
END;

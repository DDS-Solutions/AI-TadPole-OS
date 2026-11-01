-- Migration: oversight_hardened
-- Expands oversight_log to support full persistent auditing and decision tracking.

-- SQLite doesn't support complex ALTER TABLE with multiple columns or constraints well,
-- so we migrate by creating a new table and copying data (though it's likely empty).

CREATE TABLE oversight_log_new (
    id TEXT PRIMARY KEY,
    mission_id TEXT,
    agent_id TEXT NOT NULL,
    entry_type TEXT NOT NULL DEFAULT 'tool_call', -- 'tool_call' | 'capability_proposal'
    skill TEXT, -- Name of tool or proposal
    params TEXT NOT NULL, -- JSON blob for input params
    status TEXT NOT NULL, -- 'pending' | 'approved' | 'rejected'
    decision TEXT, -- 'approved' | 'rejected'
    decided_at DATETIME,
    decided_by TEXT, -- 'user' | 'system'
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    payload TEXT, -- Full JSON context for audit deep-dives
    FOREIGN KEY(mission_id) REFERENCES mission_history(id)
);

-- Copy existing data if any (mapping basic fields)
INSERT INTO oversight_log_new (id, mission_id, agent_id, skill, params, status, created_at)
SELECT id, mission_id, agent_id, skill, params, status, created_at FROM oversight_log;

DROP TABLE oversight_log;
ALTER TABLE oversight_log_new RENAME TO oversight_log;

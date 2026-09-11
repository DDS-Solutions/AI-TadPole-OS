-- Migration: Audit Trail Monotonic Sequence Ordering
-- Adds an explicit `seq INTEGER PRIMARY KEY AUTOINCREMENT` to ensure deterministic insertion ordering.

CREATE TABLE IF NOT EXISTS audit_trail_new (
    seq INTEGER PRIMARY KEY AUTOINCREMENT,
    id TEXT NOT NULL UNIQUE,
    agent_id TEXT NOT NULL,
    action TEXT NOT NULL,
    params TEXT NOT NULL,
    prev_hash TEXT NOT NULL,
    current_hash TEXT NOT NULL,
    timestamp DATETIME NOT NULL,
    signature TEXT,
    mission_id TEXT,
    user_id TEXT,
    trace_id TEXT,
    http_status INTEGER,
    cost_micros INTEGER NOT NULL DEFAULT 0,
    content TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO audit_trail_new (
    id, agent_id, action, params, prev_hash, current_hash, timestamp, signature,
    mission_id, user_id, trace_id, http_status, cost_micros, content, created_at
)
SELECT 
    id, agent_id, action, params, prev_hash, current_hash, timestamp, signature,
    mission_id, user_id, trace_id, http_status, cost_micros, content, created_at
FROM audit_trail ORDER BY rowid ASC;

DROP TABLE audit_trail;
ALTER TABLE audit_trail_new RENAME TO audit_trail;

CREATE INDEX IF NOT EXISTS idx_audit_seq ON audit_trail(seq);
CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_trail(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_agent ON audit_trail(agent_id);
CREATE INDEX IF NOT EXISTS idx_audit_mission ON audit_trail(mission_id);
CREATE INDEX IF NOT EXISTS idx_audit_user ON audit_trail(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_trace ON audit_trail(trace_id);

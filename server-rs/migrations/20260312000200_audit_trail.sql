-- Create audit_trail table for tamper-evident Merkle hash chaining
CREATE TABLE IF NOT EXISTS audit_trail (
    id TEXT PRIMARY KEY NOT NULL,
    agent_id TEXT NOT NULL,
    action TEXT NOT NULL,
    params TEXT NOT NULL,
    prev_hash TEXT NOT NULL,
    current_hash TEXT NOT NULL,
    timestamp DATETIME NOT NULL,
    signature TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_trail(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_agent ON audit_trail(agent_id);

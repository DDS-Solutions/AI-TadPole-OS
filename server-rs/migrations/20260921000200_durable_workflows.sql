-- Migration: Durable Workflows & Step Memoization (DBOS Pattern)
-- Ensures missions survive engine crashes with zero lost tokens or duplicated step side-effects.

CREATE TABLE IF NOT EXISTS durable_workflows (
    workflow_id TEXT PRIMARY KEY,
    mission_id TEXT,
    agent_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('RUNNING', 'COMPLETED', 'FAILED', 'ROLLED_BACK')),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS durable_steps (
    step_id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL REFERENCES durable_workflows(workflow_id) ON DELETE CASCADE,
    step_index INTEGER NOT NULL,
    step_name TEXT NOT NULL,
    input_hash TEXT NOT NULL,
    output_json TEXT,
    status TEXT NOT NULL CHECK(status IN ('PENDING', 'COMPLETED', 'FAILED')),
    error_detail TEXT,
    executed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(workflow_id, step_index)
);

CREATE INDEX IF NOT EXISTS idx_durable_steps_lookup ON durable_steps(workflow_id, step_index, status);
CREATE INDEX IF NOT EXISTS idx_durable_workflows_mission ON durable_workflows(mission_id);

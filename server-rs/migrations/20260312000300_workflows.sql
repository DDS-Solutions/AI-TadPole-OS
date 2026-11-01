-- Migration: Workflow Engine tables
-- Supports deterministic multi-step agent pipelines.

-- Main workflow definitions
CREATE TABLE IF NOT EXISTS workflows (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT,
    enabled         INTEGER NOT NULL DEFAULT 1,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    metadata        TEXT                     -- JSON blob for overall configuration
);

-- Workflow steps (nodes in the pipeline)
CREATE TABLE IF NOT EXISTS workflow_steps (
    id              TEXT PRIMARY KEY,
    workflow_id     TEXT NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    agent_id        TEXT NOT NULL,           -- Agent assigned to this step
    step_order      INTEGER NOT NULL,        -- Sequence in the pipeline
    name            TEXT NOT NULL,
    prompt_template TEXT NOT NULL,           -- The prompt for this step (can use variables)
    config          TEXT,                    -- JSON configuration for this step (e.g. overrides)
    UNIQUE(workflow_id, step_order)
);

-- Workflow execution history
CREATE TABLE IF NOT EXISTS workflow_runs (
    id              TEXT PRIMARY KEY,
    workflow_id     TEXT NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    started_at      TEXT NOT NULL,
    completed_at    TEXT,
    status          TEXT NOT NULL,           -- "running" | "completed" | "failed" | "cancelled"
    current_step    INTEGER NOT NULL DEFAULT 0,
    context         TEXT                     -- JSON blob accumulating state across steps
);

-- Individual step execution records within a workflow run
CREATE TABLE IF NOT EXISTS workflow_step_runs (
    id              TEXT PRIMARY KEY,
    run_id          TEXT NOT NULL REFERENCES workflow_runs(id) ON DELETE CASCADE,
    step_id         TEXT NOT NULL REFERENCES workflow_steps(id) ON DELETE CASCADE,
    mission_id      TEXT,                    -- Links to the actual agent mission
    started_at      TEXT NOT NULL,
    completed_at    TEXT,
    status          TEXT NOT NULL,           -- "completed" | "failed"
    output_text     TEXT,                    -- Captured output for the next step's context
    cost_usd        REAL NOT NULL DEFAULT 0.0
);

CREATE INDEX IF NOT EXISTS idx_workflow_steps_workflow_id ON workflow_steps(workflow_id);
CREATE INDEX IF NOT EXISTS idx_workflow_runs_workflow_id ON workflow_runs(workflow_id);
CREATE INDEX IF NOT EXISTS idx_workflow_step_runs_run_id ON workflow_step_runs(run_id);

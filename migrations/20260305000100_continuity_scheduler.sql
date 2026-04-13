-- Migration: Continuity Scheduler tables
-- Adds scheduled autonomous tasks (cron-driven missions) to Tadpole OS.

-- Main job registry
CREATE TABLE IF NOT EXISTS scheduled_jobs (
    id              TEXT PRIMARY KEY,
    agent_id        TEXT NOT NULL,
    name            TEXT NOT NULL,
    prompt          TEXT NOT NULL,
    cron_expr       TEXT NOT NULL,           -- standard 5-field cron: "0 9 * * *"
    budget_usd      REAL NOT NULL DEFAULT 0.10,
    enabled         INTEGER NOT NULL DEFAULT 1,  -- 0 = disabled
    last_run_at     TEXT,                    -- ISO-8601 UTC timestamp
    next_run_at     TEXT NOT NULL,           -- ISO-8601 UTC timestamp (pre-computed)
    consecutive_failures  INTEGER NOT NULL DEFAULT 0,
    max_failures    INTEGER NOT NULL DEFAULT 3,  -- auto-disable after N consecutive failures
    created_at      TEXT NOT NULL,
    metadata        TEXT                     -- optional JSON blob for extra fields
);

-- Run history for each job
CREATE TABLE IF NOT EXISTS scheduled_job_runs (
    id              TEXT PRIMARY KEY,
    job_id          TEXT NOT NULL REFERENCES scheduled_jobs(id) ON DELETE CASCADE,
    mission_id      TEXT,                    -- links to existing mission_history.id
    started_at      TEXT NOT NULL,
    completed_at    TEXT,
    status          TEXT NOT NULL,           -- "running" | "completed" | "failed" | "budget_exceeded" | "skipped"
    cost_usd        REAL NOT NULL DEFAULT 0.0,
    output_summary  TEXT                     -- first 500 chars of agent response
);

CREATE INDEX IF NOT EXISTS idx_scheduled_jobs_next_run ON scheduled_jobs(next_run_at, enabled);
CREATE INDEX IF NOT EXISTS idx_scheduled_job_runs_job_id ON scheduled_job_runs(job_id);

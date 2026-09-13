-- Create mission_quotas table for persistent mission-level budget management
CREATE TABLE IF NOT EXISTS mission_quotas (
    id TEXT PRIMARY KEY NOT NULL,
    cluster_id TEXT NOT NULL UNIQUE,
    budget_usd REAL NOT NULL DEFAULT 5.0,
    used_usd REAL NOT NULL DEFAULT 0.0,
    reset_period TEXT NOT NULL CHECK(reset_period IN ('daily', 'monthly', 'never')),
    last_reset_at DATETIME NOT NULL,
    next_reset_at DATETIME NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_mission_quotas_cluster ON mission_quotas(cluster_id);

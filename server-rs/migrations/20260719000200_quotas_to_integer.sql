-- Migrate agent_quotas and mission_quotas columns from REAL to INTEGER (micro-dollars).
-- SQLite does not support ALTER COLUMN, so we use the create-copy-drop-rename pattern.
-- Values are already stored as micro-dollar integers (via 20260716000100); this migration
-- enforces the INTEGER affinity at the schema level to prevent floating-point drift.

-- ── agent_quotas ──────────────────────────────────────────────────────────────
CREATE TABLE agent_quotas_new (
    id TEXT PRIMARY KEY NOT NULL,
    entity_id TEXT NOT NULL UNIQUE,
    budget_usd INTEGER NOT NULL DEFAULT 500000,
    used_usd INTEGER NOT NULL DEFAULT 0,
    reset_period TEXT NOT NULL CHECK(reset_period IN ('daily', 'monthly', 'never')),
    last_reset_at DATETIME NOT NULL,
    next_reset_at DATETIME NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO agent_quotas_new (id, entity_id, budget_usd, used_usd, reset_period, last_reset_at, next_reset_at, created_at)
    SELECT id, entity_id,
        CAST(ROUND(budget_usd) AS INTEGER),
        CAST(ROUND(used_usd) AS INTEGER),
        reset_period, last_reset_at, next_reset_at, created_at
    FROM agent_quotas;

DROP TABLE agent_quotas;
ALTER TABLE agent_quotas_new RENAME TO agent_quotas;
CREATE INDEX IF NOT EXISTS idx_agent_quotas_entity ON agent_quotas(entity_id);

-- ── mission_quotas ────────────────────────────────────────────────────────────
CREATE TABLE mission_quotas_new (
    id TEXT PRIMARY KEY NOT NULL,
    cluster_id TEXT NOT NULL UNIQUE,
    budget_usd INTEGER NOT NULL DEFAULT 5000000,
    used_usd INTEGER NOT NULL DEFAULT 0,
    reset_period TEXT NOT NULL CHECK(reset_period IN ('daily', 'monthly', 'never')),
    last_reset_at DATETIME NOT NULL,
    next_reset_at DATETIME NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO mission_quotas_new (id, cluster_id, budget_usd, used_usd, reset_period, last_reset_at, next_reset_at, created_at)
    SELECT id, cluster_id,
        CAST(ROUND(budget_usd) AS INTEGER),
        CAST(ROUND(used_usd) AS INTEGER),
        reset_period, last_reset_at, next_reset_at, created_at
    FROM mission_quotas;

DROP TABLE mission_quotas;
ALTER TABLE mission_quotas_new RENAME TO mission_quotas;
CREATE INDEX IF NOT EXISTS idx_mission_quotas_cluster ON mission_quotas(cluster_id);

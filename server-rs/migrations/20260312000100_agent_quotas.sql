-- Create agent_quotas table for persistent LLM budget management
CREATE TABLE IF NOT EXISTS agent_quotas (
    id TEXT PRIMARY KEY NOT NULL,
    entity_id TEXT NOT NULL UNIQUE,
    budget_usd REAL NOT NULL DEFAULT 1.0,
    used_usd REAL NOT NULL DEFAULT 0.0,
    reset_period TEXT NOT NULL CHECK(reset_period IN ('daily', 'monthly', 'never')),
    last_reset_at DATETIME NOT NULL,
    next_reset_at DATETIME NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_agent_quotas_entity ON agent_quotas(entity_id);

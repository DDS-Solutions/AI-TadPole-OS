-- Initial schema for Tadpole OS
-- Generated from current db.rs logic to support sqlx migrate framework

CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    role TEXT NOT NULL,
    department TEXT NOT NULL,
    description TEXT NOT NULL,
    model_id TEXT,
    tokens_used INTEGER DEFAULT 0,
    status TEXT NOT NULL,
    theme_color TEXT,
    budget_usd REAL DEFAULT 0.0,
    cost_usd REAL DEFAULT 0.0,
    metadata TEXT NOT NULL, -- JSON blob
    skills TEXT, -- JSON array
    workflows TEXT, -- JSON array
    model_2 TEXT,
    model_3 TEXT,
    model_config2 TEXT, -- JSON blob
    model_config3 TEXT, -- JSON blob
    active_model_slot INTEGER DEFAULT 1,
    voice_id TEXT,
    voice_engine TEXT
);

CREATE TABLE IF NOT EXISTS mission_history (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    budget_usd REAL DEFAULT 0.0,
    cost_usd REAL DEFAULT 0.0,
    FOREIGN KEY(agent_id) REFERENCES agents(id)
);

CREATE TABLE IF NOT EXISTS mission_logs (
    id TEXT PRIMARY KEY,
    mission_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    source TEXT NOT NULL, -- 'User' | 'System' | 'Agent'
    text TEXT NOT NULL,
    severity TEXT NOT NULL, -- 'info' | 'success' | 'warning' | 'error'
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    metadata TEXT, -- JSON blob
    FOREIGN KEY(mission_id) REFERENCES mission_history(id)
);

CREATE TABLE IF NOT EXISTS oversight_log (
    id TEXT PRIMARY KEY,
    mission_id TEXT,
    agent_id TEXT NOT NULL,
    skill TEXT NOT NULL,
    params TEXT NOT NULL, -- JSON blob
    status TEXT NOT NULL, -- 'pending' | 'approved' | 'rejected'
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(mission_id) REFERENCES mission_history(id)
);

CREATE TABLE IF NOT EXISTS swarm_context (
    id TEXT PRIMARY KEY,
    mission_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    topic TEXT NOT NULL,
    finding TEXT NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(mission_id) REFERENCES mission_history(id)
);

CREATE TABLE IF NOT EXISTS benchmarks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    test_id TEXT NOT NULL,
    mean_ms REAL NOT NULL,
    p95_ms REAL,
    p99_ms REAL,
    target_value TEXT,
    status TEXT NOT NULL,
    metadata TEXT, -- JSON blob
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

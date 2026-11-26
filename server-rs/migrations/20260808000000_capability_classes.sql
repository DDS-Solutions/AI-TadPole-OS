-- Migration: Capability Classes & Hierarchical Policies
-- Decouples installation/mutation capabilities from execution rights.

CREATE TABLE IF NOT EXISTS capability_policies (
    capability_class TEXT NOT NULL,
    resource_pattern TEXT NOT NULL,
    mode TEXT NOT NULL CHECK(mode IN ('allow', 'deny', 'prompt')),
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (capability_class, resource_pattern)
);

CREATE TABLE IF NOT EXISTS agent_capability_policies (
    agent_id TEXT NOT NULL,
    capability_class TEXT NOT NULL,
    resource_pattern TEXT NOT NULL,
    mode TEXT NOT NULL CHECK(mode IN ('allow', 'deny', 'prompt')),
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (agent_id, capability_class, resource_pattern)
);

CREATE TABLE IF NOT EXISTS role_capability_policies (
    role TEXT NOT NULL,
    capability_class TEXT NOT NULL,
    resource_pattern TEXT NOT NULL,
    mode TEXT NOT NULL CHECK(mode IN ('allow', 'deny', 'prompt')),
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (role, capability_class, resource_pattern)
);

-- Composite Indexes for High-Performance Authorization Lookups
CREATE INDEX IF NOT EXISTS idx_cap_global ON capability_policies(capability_class, resource_pattern);
CREATE INDEX IF NOT EXISTS idx_agent_cap ON agent_capability_policies(agent_id, capability_class, resource_pattern);
CREATE INDEX IF NOT EXISTS idx_role_cap ON role_capability_policies(role, capability_class, resource_pattern);

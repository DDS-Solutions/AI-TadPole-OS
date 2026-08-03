-- Sovereign Governance: Persistent Role Blueprint System
-- This table stores reusable agent configurations that can be used to boot new specialist nodes.

CREATE TABLE IF NOT EXISTS role_blueprints (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    department TEXT NOT NULL,
    description TEXT NOT NULL,
    skills TEXT NOT NULL DEFAULT '[]', -- JSON array of skill slugs
    workflows TEXT NOT NULL DEFAULT '[]', -- JSON array of workflow slugs
    mcp_tools TEXT NOT NULL DEFAULT '[]', -- JSON array of MCP tools
    requires_oversight BOOLEAN NOT NULL DEFAULT 0,
    model_id TEXT, -- Optional default model override
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Index for department-based role filtering
CREATE INDEX IF NOT EXISTS idx_role_blueprints_department ON role_blueprints(department);

-- Migration: autonomous_governance
-- Supports Goal 3 (Cognitive Autonomy) and Goal 4 (Governed Autonomy)

-- Table: permission_policies
-- Manages the dynamic whitelist/guardrail config
CREATE TABLE IF NOT EXISTS permission_policies (
    tool_name TEXT PRIMARY KEY,
    mode TEXT NOT NULL, -- 'Allow', 'Prompt', 'Deny'
    reason TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Table: capability_proposals
-- Stores agent-written code pending physicalization
CREATE TABLE IF NOT EXISTS capability_proposals (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    cap_type TEXT NOT NULL, -- 'skill', 'workflow', 'hook'
    content TEXT NOT NULL,  -- The actual JSON or Markdown code
    agent_id TEXT NOT NULL,
    mission_id TEXT,
    status TEXT NOT NULL DEFAULT 'pending', -- 'pending', 'approved', 'rejected'
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Seed initial default policies to match legacy hardcoded values
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('ls', 'Allow');
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('grep', 'Allow');
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('find', 'Allow');
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('cat', 'Allow');
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('read_file', 'Allow');
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('list_dir', 'Allow');
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('get_file_contents', 'Allow');
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('grep_search', 'Allow');
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('get_project_status', 'Allow');

-- Guardrails
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('bash', 'Prompt');
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('write_to_file', 'Prompt');
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('delete_file', 'Prompt');
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('git_push', 'Prompt');
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('deploy_application', 'Prompt');
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('propose_capability', 'Prompt');

-- Whitelist low-risk discovery tools for Silent Operation
-- This reduces "Approval Fatigue" for systemic metadata checks.
-- Note: Sensitive tools like 'read_file' or 'write_file' remain as 'Prompt' (Human-in-the-Loop).

INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('list_skill_metadata', 'allow');
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('get_agent_metrics', 'allow');
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('get_current_mission_status', 'allow');
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('update_working_memory', 'allow');
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('share_finding', 'allow');

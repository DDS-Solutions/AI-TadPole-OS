-- Create agent-specific permission policies table
CREATE TABLE IF NOT EXISTS agent_permission_policies (
    agent_id TEXT NOT NULL,
    tool_name TEXT NOT NULL,
    mode TEXT NOT NULL,
    PRIMARY KEY (agent_id, tool_name)
);

-- Create role-specific permission policies table
CREATE TABLE IF NOT EXISTS role_permission_policies (
    role TEXT NOT NULL,
    tool_name TEXT NOT NULL,
    mode TEXT NOT NULL,
    PRIMARY KEY (role, tool_name)
);

-- Insert default role permission policies
INSERT OR IGNORE INTO role_permission_policies (role, tool_name, mode) VALUES
('specialist', 'write_file', 'prompt'),
('specialist', 'delete_file', 'prompt'),
('specialist', 'execute_bash', 'prompt'),
('security', 'write_file', 'allow'),
('security', 'delete_file', 'allow'),
('security', 'execute_bash', 'allow');

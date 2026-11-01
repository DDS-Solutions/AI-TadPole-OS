-- Seed get_system_metrics read-only safe skill into permission_policies table
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('get_system_metrics', 'allow');

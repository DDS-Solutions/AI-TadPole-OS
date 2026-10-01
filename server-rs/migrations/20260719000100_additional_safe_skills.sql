-- Seed additional read-only safe skills into permission_policies table
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('list_files', 'allow');
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('read_codebase_file', 'allow');
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('list_file_symbols', 'allow');
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('get_symbol_body', 'allow');
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('search_mission_knowledge', 'allow');
INSERT OR IGNORE INTO permission_policies (tool_name, mode) VALUES ('search_global_vault', 'allow');

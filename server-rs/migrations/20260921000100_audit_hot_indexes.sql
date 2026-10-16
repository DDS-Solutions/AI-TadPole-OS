-- Hot-path composite indexes for high-throughput swarm and audit operations
-- Optimizes:
-- 1. Mission history lookups and reaping queries (updated_at, is_pinned, status)
-- 2. Oversight log querying by mission and agent
-- 3. Mission logs sequential retrieval by mission and timestamp
-- 4. Audit trail queries filtered by agent and timestamp

CREATE INDEX IF NOT EXISTS idx_mission_history_reap ON mission_history(updated_at, is_pinned);
CREATE INDEX IF NOT EXISTS idx_mission_history_agent_status ON mission_history(agent_id, status);
CREATE INDEX IF NOT EXISTS idx_oversight_log_mission ON oversight_log(mission_id);
CREATE INDEX IF NOT EXISTS idx_oversight_log_agent_status ON oversight_log(agent_id, status);
CREATE INDEX IF NOT EXISTS idx_mission_logs_mission_ts ON mission_logs(mission_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_trail_agent_ts ON audit_trail(agent_id, timestamp);

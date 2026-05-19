-- Add health monitoring and mission state persistence to agents
ALTER TABLE agents ADD COLUMN failure_count INTEGER DEFAULT 0;
ALTER TABLE agents ADD COLUMN last_failure_at DATETIME;
ALTER TABLE agents ADD COLUMN active_mission TEXT; -- JSON blob

-- Add heartbeat_at column to agents table for stale run recovery
ALTER TABLE agents ADD COLUMN heartbeat_at DATETIME;

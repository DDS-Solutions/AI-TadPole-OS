-- Add is_degraded column to mission_history table for graceful degradation tracking
ALTER TABLE mission_history ADD COLUMN is_degraded BOOLEAN DEFAULT 0;

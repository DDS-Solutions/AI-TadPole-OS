-- Add Oversight Gate column to agents table
-- This allows persistent governance per agent, enabling "Safe Mode" for all autonomous actions.

ALTER TABLE agents ADD COLUMN requires_oversight BOOLEAN DEFAULT FALSE;

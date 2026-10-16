-- Add working_memory column to agents table for structured reasoning persistence.
-- This column stores a JSON object that serves as the agent's long-term scratchpad.
ALTER TABLE agents ADD COLUMN working_memory TEXT DEFAULT '{}';

-- Fixes the missing connector_configs column in the agents table.
-- Generated to resolve the "table agents has no column named connector_configs" error.

-- 1. Add the missing connector_configs column to store JSON configurations.
ALTER TABLE agents ADD COLUMN connector_configs TEXT DEFAULT '[]';

-- 2. Optional: Cleanup legacy/erroneous field if supported (Legacy SQLite compatibility safe check)
-- ALTER TABLE agents DROP COLUMN encrypted_config; 

-- Add missing model configuration fields to the agents table
-- This ensures that model selection (provider, model_id, etc.) persists across engine restarts.

-- Note: model_id already exists from the initial schema, but we're ensuring complete parity here.
ALTER TABLE agents ADD COLUMN provider TEXT DEFAULT 'gemini'; -- Default to gemini to avoid NULL breakage
ALTER TABLE agents ADD COLUMN api_key TEXT;
ALTER TABLE agents ADD COLUMN base_url TEXT; -- New column for custom endpoints
ALTER TABLE agents ADD COLUMN system_prompt TEXT;
ALTER TABLE agents ADD COLUMN temperature REAL;

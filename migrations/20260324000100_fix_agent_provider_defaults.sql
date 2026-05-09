-- Fix agent provider mapping for known models
-- This resolves the UI discrepancy where Mercury-2 models were defaulting to 'gemini' provider

-- 1. Update Mercury-2 agents to use 'inception' provider
UPDATE agents SET provider = 'inception' WHERE model_id = 'Mercury-2';

-- 2. Update Llama agents to use 'groq' or 'meta' based on model_id
UPDATE agents SET provider = 'groq' WHERE model_id LIKE '%(Groq)%';
UPDATE agents SET provider = 'groq' WHERE model_id LIKE 'llama-3.3-70b-versatile';

-- 3. Update Phi agents to use 'local' or 'microsoft'
UPDATE agents SET provider = 'local' WHERE model_id LIKE 'Phi%';

-- 4. Update gemini-specific models to ensure they stay 'gemini'
UPDATE agents SET provider = 'gemini' WHERE model_id LIKE 'gemini%';

-- Note: We are keeping the column default for now to avoid complex SQLite table recreation, 
-- but the backend code will now be more explicit.

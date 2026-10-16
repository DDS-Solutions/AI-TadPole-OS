-- OKF: Add metadata columns to SQLite metadata table
--
-- Adds columns to knowledge_store_meta to support self-describing fields
-- specified in the Open Knowledge Format (OKF).
--
-- @docs ARCHITECTURE:IKS
-- Metadata: [IKS_PLAN]

ALTER TABLE knowledge_store_meta ADD COLUMN concept_type TEXT NOT NULL DEFAULT 'general';
ALTER TABLE knowledge_store_meta ADD COLUMN title TEXT;
ALTER TABLE knowledge_store_meta ADD COLUMN description TEXT;
ALTER TABLE knowledge_store_meta ADD COLUMN resource_uri TEXT;
ALTER TABLE knowledge_store_meta ADD COLUMN tags TEXT;

-- Lean OKF Extension (OKF v0.1.5): Add security_tier and parent_id
--
-- Adds lightweight governance tiering and parent/child lineage fields
-- to knowledge_store_meta.
--
-- @docs ARCHITECTURE:IKS
-- Metadata: [IKS_LEAN_OKF]

ALTER TABLE knowledge_store_meta ADD COLUMN security_tier TEXT NOT NULL DEFAULT 'BRONZE_ADHOC';
ALTER TABLE knowledge_store_meta ADD COLUMN parent_id TEXT;

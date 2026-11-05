-- IKS: Add text column to SQLite metadata table
--
-- Moves the `text` content from LanceDB-only storage into SQLite.
-- This fixes `get_by_id` always returning empty text, and eliminates the
-- need to perform a LanceDB predicate scan for point-lookups.
--
-- Rationale: IKS entries are curated short facts — storing text in SQLite is
-- the most performant choice (no vector scan needed for ID lookups).
--
-- @docs ARCHITECTURE:IKS
-- Metadata: [IKS_PLAN]

ALTER TABLE knowledge_store_meta ADD COLUMN text TEXT NOT NULL DEFAULT '';

-- Institutional Knowledge Store: SQLite metadata index
-- The LanceDB vector store (knowledge_store table) is created lazily at runtime.
-- This table provides fast pre-filtering by topic, cluster, TTL, and dedup.
--
-- @docs ARCHITECTURE:IKS
-- Metadata: [IKS_PLAN]

CREATE TABLE IF NOT EXISTS knowledge_store_meta (
    -- Primary identity
    id                TEXT PRIMARY KEY,           -- UUID, matches LanceDB entry id
    content_hash      TEXT NOT NULL UNIQUE,       -- SHA-256(text) — dedup + P2P conflict key

    -- Semantic classification
    topic             TEXT NOT NULL DEFAULT 'general',  -- e.g. 'finance', 'agent_pattern', 'sop'
    cluster_id        TEXT,                       -- NULL = global; set = cluster-scoped
    source_node_id    TEXT,                       -- NULL = local; set = written by remote Bunker (P2P)
    source_agent_id   TEXT,                       -- Which agent authored this entry

    -- Quality signals
    confidence        REAL NOT NULL DEFAULT 1.0,  -- 0.0–1.0, decays over time
    access_count      INTEGER NOT NULL DEFAULT 0, -- Hit counter for relevance boosting
    last_accessed_at  INTEGER,                    -- Unix timestamp of last retrieval

    -- Lifecycle (Q3: agent default = 90d, human-confirmed = NULL/never)
    ttl               INTEGER,                    -- Unix expiry timestamp; NULL = never expires
    created_at        INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at        INTEGER NOT NULL DEFAULT (unixepoch()),

    -- Provenance (for P2P and audit)
    human_confirmed   INTEGER NOT NULL DEFAULT 0  -- 1 if a human approved this entry
);

-- Indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_ks_topic        ON knowledge_store_meta(topic);
CREATE INDEX IF NOT EXISTS idx_ks_cluster      ON knowledge_store_meta(cluster_id);
CREATE INDEX IF NOT EXISTS idx_ks_ttl          ON knowledge_store_meta(ttl) WHERE ttl IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_ks_hash         ON knowledge_store_meta(content_hash);
CREATE INDEX IF NOT EXISTS idx_ks_source_node  ON knowledge_store_meta(source_node_id) WHERE source_node_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_ks_confidence   ON knowledge_store_meta(confidence);
CREATE INDEX IF NOT EXISTS idx_ks_created_at   ON knowledge_store_meta(created_at);

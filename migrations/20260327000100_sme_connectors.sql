-- Phase 2: SME Data Connectors & Background Sync
-- Creates the manifest to track incremental updates from external sources (Slack, Notion, FS)

CREATE TABLE IF NOT EXISTS sync_manifest (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    source_type TEXT NOT NULL, -- 'slack', 'notion', 'fs', etc.
    source_uri TEXT NOT NULL,  -- file path, channel id, or page id
    last_sync_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    checksum TEXT,             -- Optional: for detecting file changes
    status TEXT DEFAULT 'idle', -- 'idle', 'syncing', 'error'
    metadata TEXT,             -- JSON blob for source-specific state (e.g. pagination cursors)
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
);

-- Index for fast lookup of pending syncs
CREATE INDEX idx_sync_manifest_agent ON sync_manifest(agent_id);
CREATE INDEX idx_sync_manifest_status ON sync_manifest(status);

-- Add encrypted_config to agents for secure connector storage
-- Note: 'agents' table exists from initial schema, so we ALTER
ALTER TABLE agents ADD COLUMN encrypted_config BLOB;

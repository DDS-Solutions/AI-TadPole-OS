-- Add Lean Workspace File Revision Ledger table
-- Stores pre-mutation SHA-256 content hashes, BLOB payload, and revision history for workspace files

CREATE TABLE IF NOT EXISTS file_revisions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id TEXT NOT NULL,          -- Workspace root path or identifier
    file_path TEXT NOT NULL,             -- Relative file path
    hash TEXT NOT NULL,                  -- SHA-256 hash of file content
    content BLOB NOT NULL,               -- Binary BLOB payload (supports all file types)
    size_bytes INTEGER NOT NULL,         -- Uncompressed byte size
    version_num INTEGER NOT NULL,        -- Monotonically increasing revision index (0, 1, 2...)
    mission_id TEXT,                     -- Associated agent mission ID
    agent_id TEXT,                       -- Agent that made the modification
    created_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_file_revisions_unique 
ON file_revisions (workspace_id, file_path, version_num);

CREATE INDEX IF NOT EXISTS idx_file_revisions_lookup 
ON file_revisions (workspace_id, file_path, version_num DESC);

-- Create system_backups table for durable state backup tracking & auditability
CREATE TABLE IF NOT EXISTS system_backups (
    id TEXT PRIMARY KEY,
    snapshot_filename TEXT NOT NULL,
    destination_type TEXT NOT NULL CHECK(destination_type IN ('local', 'cloud_sync')),
    destination_path TEXT NOT NULL,
    file_size_bytes INTEGER NOT NULL,
    sha256_checksum TEXT NOT NULL,
    sqlite_bytes INTEGER NOT NULL,
    lancedb_bytes INTEGER NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('completed', 'failed', 'pruned')),
    error_message TEXT,
    created_at INTEGER NOT NULL,
    pruned_at INTEGER
);

CREATE INDEX IF NOT EXISTS idx_system_backups_created_at ON system_backups(created_at);

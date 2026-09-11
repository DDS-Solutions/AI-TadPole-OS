-- Add telemetry columns to sync_manifest for high-fidelity reporting
-- Allows the Workspace Manager to show real file counts and byte sizes.

ALTER TABLE sync_manifest ADD COLUMN file_count INTEGER DEFAULT 0;
ALTER TABLE sync_manifest ADD COLUMN total_bytes INTEGER DEFAULT 0;

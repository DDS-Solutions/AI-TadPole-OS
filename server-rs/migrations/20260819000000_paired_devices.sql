-- Migration: Create paired_devices table for Zero-Trust Remote Companion Persistence
CREATE TABLE IF NOT EXISTS paired_devices (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    user_name TEXT NOT NULL,
    public_key TEXT NOT NULL,
    paired_at TEXT NOT NULL,
    status TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_paired_devices_status ON paired_devices(status);

-- Migration: Telemetry Schema Hardening
-- Enforces integer micro-dollar cost attribution, trace linkage, HTTP status tracking, and payload content on audit logs.

ALTER TABLE audit_trail ADD COLUMN trace_id TEXT;
ALTER TABLE audit_trail ADD COLUMN http_status INTEGER;
ALTER TABLE audit_trail ADD COLUMN cost_micros INTEGER NOT NULL DEFAULT 0;
ALTER TABLE audit_trail ADD COLUMN content TEXT;

CREATE INDEX IF NOT EXISTS idx_audit_trace ON audit_trail(trace_id);

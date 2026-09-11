-- Migration: Add version column for optimistic locking on workflows.
-- Prevents concurrent step mutations from corrupting in-flight workflow runs.

ALTER TABLE workflows ADD COLUMN version INTEGER NOT NULL DEFAULT 1;

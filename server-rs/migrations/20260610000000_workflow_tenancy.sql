-- Migration: Add multi-tenancy columns to workflows and workflow runs.
-- All existing records default to the 'default' tenant.

ALTER TABLE workflows ADD COLUMN tenant_id TEXT NOT NULL DEFAULT 'default';
ALTER TABLE workflow_runs ADD COLUMN tenant_id TEXT NOT NULL DEFAULT 'default';

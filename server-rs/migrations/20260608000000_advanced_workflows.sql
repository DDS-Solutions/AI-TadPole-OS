-- Migration: Add metadata support to workflow step runs
ALTER TABLE workflow_step_runs ADD COLUMN metadata TEXT;

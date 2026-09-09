-- Migration: Add workflow_id to scheduled_jobs
-- Allows the scheduler to trigger either an agent mission OR a complex workflow.

ALTER TABLE scheduled_jobs ADD COLUMN workflow_id TEXT REFERENCES workflows(id) ON DELETE SET NULL;

-- Ensure either agent_id/prompt is present OR workflow_id is present
-- SQLite doesn't support adding CHECK constraints via ALTER TABLE easily without recreating the table,
-- but logic-wise: if workflow_id is set, it overrides the individual agent mission.

-- Migration: Add retry policies and dependency tracking (DAG) to workflow steps.
-- Default retry count is 3. Default backoff is 2 seconds.
-- depends_on defaults to empty JSON array '[]'.

ALTER TABLE workflow_steps ADD COLUMN max_retries INTEGER DEFAULT 3;
ALTER TABLE workflow_steps ADD COLUMN backoff_factor_secs INTEGER DEFAULT 2;
ALTER TABLE workflow_steps ADD COLUMN depends_on TEXT DEFAULT '[]';

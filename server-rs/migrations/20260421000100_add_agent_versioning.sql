-- Add versioning support for Optimistic Concurrency Control (OCC)
-- This prevents the "Lost Update" problem during high-concurrency swarm operations.
ALTER TABLE agents ADD COLUMN version INTEGER DEFAULT 1;

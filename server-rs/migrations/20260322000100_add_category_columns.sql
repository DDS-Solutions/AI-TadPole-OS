-- Migration: Add category columns to agents and workflows
-- Supports the "User" vs "AI" split in the Swarm Manager and Skills UI.

ALTER TABLE agents ADD COLUMN category TEXT DEFAULT 'user';
ALTER TABLE workflows ADD COLUMN category TEXT DEFAULT 'user';

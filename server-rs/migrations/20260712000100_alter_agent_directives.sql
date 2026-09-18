-- Migration: 20260712000100_alter_agent_directives.sql
-- Add reasoning_trace and artifacts columns to agent_directives table for standard A2A compliance

ALTER TABLE agent_directives ADD COLUMN reasoning_trace TEXT;
ALTER TABLE agent_directives ADD COLUMN artifacts TEXT;

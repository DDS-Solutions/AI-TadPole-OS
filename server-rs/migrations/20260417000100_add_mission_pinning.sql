-- Phase 3: Swarm Architectural Hardening
-- Adds mission pinning support to protect critical reasoning chains from the Swarm Reaper.

-- 1. Add is_pinned column with default FALSE
-- SEC: Missions are ephemereal by default (48h lifecycle); explicit pinning is required for retention.
ALTER TABLE mission_history ADD COLUMN is_pinned BOOLEAN NOT NULL DEFAULT 0;

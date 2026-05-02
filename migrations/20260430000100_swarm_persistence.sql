-- Add swarm persistence tables for Phase 3/4
CREATE TABLE IF NOT EXISTS agent_directives (
    id TEXT PRIMARY KEY NOT NULL,
    mission_id TEXT NOT NULL,
    source_agent_id TEXT NOT NULL,
    target_agent_id TEXT NOT NULL,
    instruction TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    result TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS peer_reviews (
    id TEXT PRIMARY KEY NOT NULL,
    mission_id TEXT NOT NULL,
    requester_id TEXT NOT NULL,
    reviewer_id TEXT NOT NULL,
    content_to_review TEXT NOT NULL,
    criteria TEXT,
    feedback TEXT,
    status TEXT NOT NULL DEFAULT 'requested',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

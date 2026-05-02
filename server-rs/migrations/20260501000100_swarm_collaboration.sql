-- Swarm Collaboration: Inter-agent tasking and auditing.

CREATE TABLE IF NOT EXISTS agent_directives (
    id TEXT PRIMARY KEY,
    mission_id TEXT NOT NULL,
    source_agent_id TEXT NOT NULL,
    target_agent_id TEXT NOT NULL,
    instruction TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending', -- pending, acknowledged, completed, failed
    result TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(mission_id) REFERENCES mission_history(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS peer_reviews (
    id TEXT PRIMARY KEY,
    mission_id TEXT NOT NULL,
    requester_id TEXT NOT NULL,
    reviewer_id TEXT NOT NULL,
    content_to_review TEXT NOT NULL,
    criteria TEXT,
    feedback TEXT,
    status TEXT NOT NULL DEFAULT 'requested', -- requested, reviewed, rejected
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(mission_id) REFERENCES mission_history(id) ON DELETE CASCADE
);

-- Indices for performance
CREATE INDEX IF NOT EXISTS idx_directives_target ON agent_directives(target_agent_id, status);
CREATE INDEX IF NOT EXISTS idx_reviews_reviewer ON peer_reviews(reviewer_id, status);

-- Swarm Graph Foundation: Mission Relationships
-- Formulates the many-to-many relationship mapping for agents and mission dependencies.

CREATE TABLE IF NOT EXISTS mission_relationships (
    id TEXT PRIMARY KEY NOT NULL,
    from_id TEXT NOT NULL,         -- Source (Agent or Mission ID)
    to_id TEXT NOT NULL,           -- Target (Agent or Mission ID)
    relationship_type TEXT NOT NULL, -- BLOCKS, DEPENDS_ON, ASSIGNED_TO, CREATED_BY
    metadata TEXT,                 -- JSON context for the relationship
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexing for fast graph traversal
CREATE INDEX IF NOT EXISTS idx_mission_from ON mission_relationships(from_id);
CREATE INDEX IF NOT EXISTS idx_mission_to ON mission_relationships(to_id);
CREATE INDEX IF NOT EXISTS idx_mission_rel_type ON mission_relationships(relationship_type);

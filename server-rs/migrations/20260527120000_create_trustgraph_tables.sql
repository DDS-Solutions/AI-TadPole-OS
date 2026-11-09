-- SQL Migration: TrustGraph Nodes & Relations Schema
CREATE TABLE IF NOT EXISTS graph_entities (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL,          -- 'AGENT', 'MISSION', 'CONCEPT', 'FILE', 'OUTCOME'
    description TEXT,
    mission_id TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE IF NOT EXISTS graph_relations (
    id TEXT PRIMARY KEY,
    source_entity_id TEXT NOT NULL,
    target_entity_id TEXT NOT NULL,
    relation_type TEXT NOT NULL,  -- 'DEPENDS_ON', 'CREATED', 'MUTATED', 'RESOLVED', 'COMMUNICATED'
    weight REAL NOT NULL DEFAULT 1.0,
    mission_id TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY(source_entity_id) REFERENCES graph_entities(id) ON DELETE CASCADE,
    FOREIGN KEY(target_entity_id) REFERENCES graph_entities(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_relations_source ON graph_relations(source_entity_id);
CREATE INDEX IF NOT EXISTS idx_relations_target ON graph_relations(target_entity_id);
CREATE INDEX IF NOT EXISTS idx_entities_mission ON graph_entities(mission_id);

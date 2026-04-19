//! @docs ARCHITECTURE:Intelligence
//! 
//! ### AI Assist Note
//! **Swarm Graph Validation (Relational Tests)**: Orchestrates the 
//! verification of the relational intelligence and mission 
//! dependencies for the Tadpole OS engine. Features **Mission 
//! Relationship Mapping**: ensures that `RelationshipType` 
//! (e.g., `Blocks`, `Subtask`) is correctly persisted and retrieved 
//! in the `SwarmGraph` structure. Implements **Memory Isolation**: 
//! uses in-memory SQLite pools (`sqlite::memory:`) to ensure zero 
//! side-effects during systemic intelligence audits. AI agents should 
//! run these tests to verify that the graph visualization and 
//! dependency resolution logic correctly handles complex mission 
//! chains (INT-03).
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Schema mismatch in the in-memory pool, edge 
//!   directionality errors in the graph builder, or orphan nodes 
//!   causing visualization breaks.
//! - **Trace Scope**: `server-rs::agent::tests_graph`

use crate::agent::mission::{add_mission_relationship, get_swarm_graph, create_mission};
use crate::agent::types::RelationshipType;
use sqlx::SqlitePool;

#[tokio::test]
async fn test_swarm_graph_generation() -> anyhow::Result<()> {
    // SEC-03: Use in-memory SQLite for test isolation
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    
    // Setup required tables for the test context
    sqlx::query("CREATE TABLE agents (id TEXT PRIMARY KEY, name TEXT, role TEXT, department TEXT, description TEXT, status TEXT, metadata TEXT)").execute(&pool).await?;
    sqlx::query("CREATE TABLE mission_history (id TEXT PRIMARY KEY, agent_id TEXT, title TEXT, status TEXT, budget_usd REAL, cost_usd REAL, created_at DATETIME, updated_at DATETIME, is_degraded BOOLEAN)").execute(&pool).await?;
    sqlx::query("CREATE TABLE mission_relationships (id TEXT PRIMARY KEY, from_id TEXT, to_id TEXT, relationship_type TEXT, metadata TEXT, created_at DATETIME)").execute(&pool).await?;

    // 1. Setup Agents and Missions
    let agent_id = "agent-1";
    sqlx::query("INSERT INTO agents (id, name, role, department, description, status) VALUES ('agent-1', 'Agent One', 'Developer', 'Engineering', 'Test Agent', 'idle')").execute(&pool).await?;
    
    let m1 = create_mission(&pool, agent_id, "Mission Alpha", 10.0).await?;
    let m2 = create_mission(&pool, agent_id, "Mission Beta", 10.0).await?;

    // 2. Add Relationship (M1 blocks M2)
    add_mission_relationship(&pool, &m1.id, &m2.id, RelationshipType::Blocks, None).await?;

    // 3. Verify graph data mapping to SwarmGraph structure
    let graph = get_swarm_graph(&pool).await?;
    
    assert_eq!(graph.nodes.len(), 3, "Should have 1 agent node + 2 mission nodes");
    assert_eq!(graph.edges.len(), 1, "Should have 1 relationship edge");
    
    let edge = &graph.edges[0];
    assert_eq!(edge.source, m1.id);
    assert_eq!(edge.target, m2.id);
    assert_eq!(edge.label, "blocks");

    Ok(())
}

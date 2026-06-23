# Tadpole OS — API Reference

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Documentation**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Automated governance and architectural tracking.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

**Version**: 1.1.209

**Generated**: 2026-06-23 18:40:22
Welcome to the official API reference for the Tadpole OS Sovereign Engine. All endpoints require a valid `NEURAL_TOKEN` provided via the `Authorization: Bearer <token>` header.

## Endpoints

### GetAgents

- **Endpoint**: `GET /v1/agents`
- **Handler**: `pub async fn get_agents` in `agent.rs`

Retrieves the list of all registered agents in the swarm. Implements
 HATEOAS-compliant pagination to allow for efficient UI rendering and discovery.

 ### 🛰️ Registry Introspection
 This handler pulls directly from the engine's memory-mapped `AgentRegistry`.
 It maps raw back-end models into a clean, RESTful representation for
 dashboard consumption.

---

### GetAgentMemory

- **Endpoint**: `GET /v1/agents/:agent_id/memories`
- **Handler**: `pub async fn get_agent_memory` in `memory.rs`

Retrieves semantic memories for a specific agent by scanning its local
 workspace's LanceDB vector store.

---

### GetAgent

- **Endpoint**: `GET /v1/agents/:id`
- **Handler**: `pub async fn get_agent` in `agent.rs`

Retrieves the detailed state of a specific agent by its unique identifier.
 Provides O(1) discovery for high-density swarms.

---

### GetBlastRadius

- **Endpoint**: `GET /v1/intelligence/blast-radius`
- **Handler**: `pub async fn get_blast_radius` in `intelligence.rs`

Calculates the downstream impact of changing a specific symbol.

---

### GetCodeGraph

- **Endpoint**: `GET /v1/intelligence/graph`
- **Handler**: `pub async fn get_code_graph` in `intelligence.rs`

Returns the full high-fidelity symbol graph for visualization.

---


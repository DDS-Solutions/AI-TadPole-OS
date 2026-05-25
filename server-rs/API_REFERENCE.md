> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[API_REFERENCE]` in audit logs.
>
> ### AI Assist Note
> Tadpole OS — API Reference
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# Tadpole OS — API Reference

**Generated**: 2026-04-19 22:32:08
**Version**: 1.1.17 (Sovereign Hardened)

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


[//]: # (Metadata: [API_REFERENCE])

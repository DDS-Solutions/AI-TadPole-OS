# Tadpole OS — API Reference

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Core Docs / API_REFERENCE
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py`

**Version**: 1.1.462

**Generated**: 2026-08-25 17:07:36
Welcome to the official API reference for the Tadpole OS Sovereign Engine. Protected endpoints require a valid `NEURAL_TOKEN` provided via the `Authorization: Bearer <token>` header. The public outward agent-card and catalog-search endpoints are token-free and enforce a 60-request-per-minute IP fixed window.

## Endpoints

### ImportOutwardCatalog

- **Endpoint**: `POST /a2a/v1/catalog/import`
- **Handler**: `pub async fn import_catalog_handler` in `outward_routes.rs`

Validates and atomically imports CSV or QuickBooks JSON payloads up to 512 KiB.

---

### SearchOutwardCatalog

- **Endpoint**: `GET /a2a/v1/catalog/search`
- **Handler**: `pub async fn search_catalog_handler` in `outward_routes.rs`

Searches the public customer catalog with a bounded result limit. No bearer token is required.

---

### GetOutwardAgentCard

- **Endpoint**: `GET /a2a/v1/company-agent-card.json`
- **Handler**: `pub async fn get_agent_card_handler` in `outward_routes.rs`

Returns the public A2A agent card. No bearer token is required; requests are IP rate limited.

---

### UpdateOutwardProfile

- **Endpoint**: `PUT /a2a/v1/profile`
- **Handler**: `pub async fn update_profile_handler` in `outward_routes.rs`

Updates the outward business profile, model profile, or advertised skills.

---

### GetAgents

- **Endpoint**: `GET /v1/agents`
- **Handler**: `pub async fn get_agents` in `agent.rs`

Retrieves the list of all registered agents in the swarm. Implements
 HATEOAS-compliant pagination to allow for efficient UI rendering and discovery.

 ### 🛰️ Registry Introspection
 This handler pulls directly from the engine's memory-mapped `AgentResponse`.
 It maps raw back-end models into a clean, RESTful representation for
 dashboard consumption.

---

### GetAgent

- **Endpoint**: `GET /v1/agents/:id`
- **Handler**: `pub async fn get_agent` in `agent.rs`

Retrieves the detailed state of a specific agent by its unique identifier.
 Provides O(1) discovery for high-density swarms.

---


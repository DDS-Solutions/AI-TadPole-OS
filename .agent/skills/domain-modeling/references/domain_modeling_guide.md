> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / domain-modeling
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Ubiquitous language drift, semantic naming mismatches across stack boundaries, unrecorded domain entity mutations.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[DOMAIN_MODELING]`)

# Domain Modeling & Ubiquitous Language Guide (L3)

> Comprehensive reference guide for Domain-Driven Design (DDD), bounded contexts, ubiquitous language parity across stack tiers, and glossary governance.

---

## 🏛️ 1. Sovereign Bounded Contexts

Tadpole OS enforces strict semantic boundaries across 4 core bounded contexts:

```
┌──────────────────────────────────────────────────────────────┐
│                  BOUNDED CONTEXT TAXONOMY                    │
├───────────────────────────────┬──────────────────────────────┤
│ 1. Governance & Swarm Context │ 2. Execution & Engine Context│
│ - Agent, AgentDirective       │ - Mission, WorkflowStep      │
│ - RoleBlueprint, Capability   │ - SwarmGraph, PulseTelemetry │
│ - OversightGate, PeerReview   │ - RunnerHandle, AbortSignal  │
├───────────────────────────────┼──────────────────────────────┤
│ 3. Security & Ledger Context  │ 4. Neural Memory Context     │
│ - VaultCredential, KeyStore   │ - InstitutionalKnowledgeStore│
│ - MicroQuota, TokenMeter      │ - DenseEmbedding, LanceTable │
│ - AuditTrail, SignedManifest  │ - MemoryChunk, ContextSlice  │
└───────────────────────────────┴──────────────────────────────┘
```

---

## 📐 2. Domain Entity Taxonomy (DDD)

| DDD Classification | Characteristics | Tadpole OS Examples |
|---|---|---|
| **Aggregate Root** | Primary entity controlling consistency boundary; has global identity. | [`AppState`](file:///D:/TadpoleOS-Dev/docs/GLOSSARY.md#appstate), `Agent`, `Mission` |
| **Domain Entity** | Has unique identity within an aggregate, mutable lifecycle. | `WorkflowStep`, `OversightEntry`, `AuditRecord` |
| **Value Object** | Immutable, identified solely by attribute values; zero lifecycle. | `MicroQuota`, `Ed25519Signature`, `TokenUsage`, `SemVer` |
| **Domain Service** | Stateless operation coordinating multiple aggregates. | `GovernanceHub`, `SecurityHub`, `CommunicationHub` |

---

## 🔄 3. Cross-Stack Ubiquitous Language Mapping Rule

A domain concept **MUST** maintain identical nomenclature across all architectural layers:

```
Cross-Stack Parity Invariant:
├── Rust Domain Struct    ➔ `server-rs/src/models/mission.rs` (`pub struct Mission`)
├── OpenAPI Contract      ➔ `docs/openapi.yaml` (`components.schemas.Mission`)
├── TypeScript Interface  ➔ `src/types/agent.ts` (`export interface Mission`)
├── UI State Store        ➔ `src/stores/missionStore.ts` (`useMissionStore`)
└── SQLite Persistence    ➔ `server-rs/migrations/*.sql` (`CREATE TABLE missions (...)`)
```

> ⚠️ **Semantic Divergence Anti-Pattern**: Never introduce ad-hoc local aliases across layers. If an autonomous objective is named `Mission` in Rust, it must not be labeled `Job` in the database, `TaskRun` in the frontend, or `Chore` in documentation.

---

## 📖 4. Glossary Upkeep SOP (`docs/GLOSSARY.md`)

When introducing a new Aggregate Root, Hub, or Core Entity:
1. **Define the Canonical Term**: Use PascalCase for entities (`RoleBlueprint`) and SCREAMING_SNAKE_CASE for global constants (`TELEMETRY_TX`).
2. **Assign Architectural Tier**:
   - `[Tier 1: Core]` (Kernel, Axum router, IPC bridge)
   - `[Tier 2: Admin / Engine]` (Swarm orchestration, runners, state hubs)
   - `[Tier 3: Security & Ledger]` (Vault, crypto proofs, audit trails)
3. **Register in `docs/GLOSSARY.md`**: Follow the exact schema:
   ```markdown
   ### EntityName
   [Tier X: Subsystem]
   Type: struct | enum | static | trait
   Purpose: Deterministic single-sentence definition of role and invariants.
   Parameters:
   - field_name (`Type`): Definition and invariants.
   Return Value: Output contract or N/A.
   Side Effects: State mutations, channel broadcasts, or database writes.
   Failure Conditions: Explicit panic or error return vectors.
   ```
4. **Validate with Script**: Run `python .agent/skills/domain-modeling/scripts/domain_glossary_validator.py docs/GLOSSARY.md`.

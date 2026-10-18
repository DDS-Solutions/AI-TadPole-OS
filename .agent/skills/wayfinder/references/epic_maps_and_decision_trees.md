# Fog-of-War Epic Maps & Decision Tree Schemas

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / wayfinder / references
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

## 1. The Fog-of-War Frontier Model

```
[SETTLED DECISIONS] ────> [ACTIVE DECISION TICKET] ────> [NOT YET SPECIFIED (FOG)] ────> [OUT OF SCOPE]
 Archived in Map           Resolved in current session    Hazy upcoming questions          Permanently excluded
```

### The Frontier Rules:
1. **Advance One Decision at a Time**: Never resolve multiple architectural forks in a single prompt turn.
2. **Graduate from Fog**: As an active decision settles, hazy questions in *Not Yet Specified* become crisp and graduate into new decision tickets.
3. **Immutability of Settled Decisions**: Once a decision is recorded under *Decisions So Far*, downstream tickets must build on it unless explicitly revised through an ADR.

---

## 2. Canonical Wayfinder Map Format (`.tmp/wayfinder-map-<epic>.md`)

```markdown
# Epic Map: [Epic Name]
- **Destination**: [1-2 sentence definition of completed destination]
- **Last Updated**: YYYY-MM-DD
- **Active Focus**: [Current ticket being decided]

---

## 1. Architectural Constraints & Preferences
- Invariants that govern every decision in this epic.

## 2. Settled Decisions
- [x] **[Decision 1: Database Storage Engine]**: Selected SQLite with WAL mode over PostgreSQL due to embedded zero-network requirement.

## 3. Active Decision Ticket
- [ ] **[Decision 2: State Synchronization Model]**: Evaluating WebSocket broadcast vs Server-Sent Events (SSE).

## 4. Fog of War (Not Yet Specified)
- How will clients authenticate over local LAN sockets?
- What is the backup and snapshot compaction cadence?

## 5. Out of Scope (Non-Goals)
- Cloud-hosted multi-tenant SaaS architecture.
```

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / documentation-templates
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Undocumented architectural drift, non-standard ADR structures, broken AI context headers.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[DOCS]`)

# Architecture Decision Records (ADR) & Documentation Templates (L3)

> Standardized blueprints for Architecture Decision Records (ADRs), `llms.txt` context catalogs, and multi-language AI Context Alignment headers.

---

## 🏛️ 1. Architecture Decision Record (ADR) Template

Store all architectural decision records in `docs/adr/ADR-00X-title.md`:

```markdown
# ADR-00[X]: [Title of Architectural Decision]

## Status
[Proposed | Accepted | Deprecated | Superseded by ADR-00Y]

## Context & Problem Statement
What is the specific architectural dilemma or operational requirement?
Which alternative solutions, frameworks, or database engines were evaluated?
What constraints (latency, zero-trust security, air-gapped support, bundle size) influenced this decision?

## Decision
What is the definitive technical decision and design rationale?
Which components or modules are affected?

## Consequences
- **Positive**: [Systemic performance gains, decoupling, reduced cognitive load]
- **Negative / Trade-offs**: [Incurred technical debt, operational overhead, migration burden]
- **Observability Impact**: [Required telemetry tags, OpenTelemetry spans, or parity checks]

## Verification & Compliance Gate
How will this decision be verified deterministically?
- [ ] Test suite: `npm run test:invariants:fast`
- [ ] Verification command: `cargo check` or `python execution/parity_guard.py`
```

---

## 🗺️ 2. Canonical `llms.txt` Context Map Template

The `llms.txt` standard provides an index for external AI models and developer agents to rapidly discover repository context without reading entire file trees:

```markdown
# [Project Name] AI Context Catalog

> [One-sentence sovereign mission statement summarizing repository architecture]

## Core Documentation
- [Architecture Overview](../../../../docs/ARCHITECTURE.md): High-level multi-layer system architecture.
- [Security Model](../../../../docs/Security_Model.md): Zero-Trust boundaries, Vault PBKDF2 encryption, and RBAC.
- [API Contract](../../../../docs/API_CONTRACT.md): Axum REST routes and WebSocket event contracts.
- [Getting Started](../../../../docs/GETTING_STARTED.md): Setup, configuration, and developer environment boot.

## Key Source Hubs
- `server-rs/src/routes/`: Axum HTTP and WebSocket handlers.
- `server-rs/migrations/`: Monotonic SQLite schema migrations.
- `src/stores/`: Frontend Zustand state stores and UI portals.
- `execution/`: Deterministic Python automation scripts.

## Optional Deep References
- **LLMS Full Catalog** (`llms-full.txt`): Concatenated API specifications and complete system documentation.
```

---

## 🛡️ 3. Sovereign AI Context Alignment Headers (ADG v2)

All production code files must maintain an AI Context block at the top of the file.

### Rust (`server-rs/src/**/*.rs`):
```rust
//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Core Subsystem / module_name
//! - **Primary Entrypoints**: `entrypoint_function`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Deterministic internal state integrity.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[TAG]`
```

### TypeScript / React (`src/**/*.ts`, `src/**/*.tsx`):
```typescript
/**
 * @docs ARCHITECTURE:Frontend:Stores
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI State Management / useSessionStore
 * - **Primary Entrypoints**: `useSessionStore`, `selectActiveSession`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Zero persistent state leaks outside declared storage bounds.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[SESSION_STORE]`
 */
```

### Python Scripts (`execution/*.py`, `.agent/skills/*/scripts/*.py`):
```python
#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Registry:Skills

### AI Context Alignment
- **Subsystem**: Agent Skills / skill_name / tool_name
- **Primary Entrypoints**: `run_analysis`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[tool_name]`, `[PASS]`, `[FAIL]`
- **Witness Tests**: none declared
"""
```

> ⚠️ **ADG v2 Non-Negotiable**: Never author hand-written `Witness Tests:` or `Invariants:` claims inside source files under `src/`! Those are flagged by `npm run adg:lint`. Test witness relationships are derived automatically by `npm run adg:generate` into `adg.manifest.json`.
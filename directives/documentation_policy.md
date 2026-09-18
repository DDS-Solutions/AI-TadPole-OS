> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Core Directives / documentation_policy
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[documentation_policy]`)

# Documentation Policy: Sovereign Semantic Standards

This directive establishes the mandatory protocols for maintaining documentation integrity, semantic clarity, and codebase parity across the Tadpole OS ecosystem.

## 1. The Glossary-First Rule
The `GLOSSARY.md` is the **Canonical Source of Truth** for all terminology. 
- **Requirement**: Before a new technical term, feature name, or architectural concept is introduced into the Operations Manual or README, it MUST be defined in the Glossary.
- **Goal**: Prevent information drift and ensure every term has a single, authoritative definition linked to its technical implementation.

## 2. Standardized Glossary Entry Format
Every entry in `GLOSSARY.md` must follow this high-fidelity format:

```markdown
### [Term Name]
**(Domain: [Category])**  
[A clear, concise definition of the concept.]

**Implementation Hook**: `[File Basename] / [Method or Endpoint]`
```

### 2.1 Domain Categories
Terms must be assigned to one of the following domains:
- **Governance**: Policy, oversight, and lifecycle management.
- **Intelligence**: AI reasoning, agents, and personas.
- **Memory**: RAG, databases, and knowledge persistence.
- **Security**: Encryption, auditing, and defensive protocols.
- **Performance**: Latency, swarms, and runtime metrics.
- **Skills**: Tools, MCP, and workflows.
- **Infrastructure**: API, backend processes, and file-system architecture.
- **UI**: Dashboard components and user interaction layers.

## 3. The [DEFINE:] Protocol (Tagging)
To improve navigability and signal the presence of canonical definitions, authors should use the `[DEFINE:]` tag in the Operations Manual for newly introduced or critical jargon.

- **Syntax**: `Term Name [DEFINE: Glossary Term Name]`
- **Example**: `Alpha Node [DEFINE: Alpha Node]`
- **Usage**: Use this tag when a term first appears in a section or when immediate clarification is required for complex workflows.

## 4. Documentation Code Parity (EC-01)
Documentation must reflect the *actual* state of the code.
- **Parity Guard**: The `parity_guard.py` script automatically verifies that Rust routes and OpenAPI specs match the documentation.
- **Build Failure**: Builds will be blocked if critical mismatch errors are detected by the Parity Guard.

## 5. Maintenance Checklist
- [ ] Has every new term been added to `GLOSSARY.md`?
- [ ] Are all "Implementation Hooks" pointing to the correct files and methods?
- [ ] Have `[DEFINE:]` tags been applied to high-stakes sections of the manual?
- [ ] Does the `parity_guard.py` scan pass without warnings?
- [ ] Have all directive files referencing `IDENTITY.md` by version been updated to `v1.2.1`?

## 6. Directive Version Synchronization Policy
Any directive file that references `IDENTITY.md` by version string (e.g., `IDENTITY.md v1.2.1`) **must** be updated every time `IDENTITY.md` is versioned.
- **Enforcement**: `parity_guard.py` must scan for stale version strings (e.g., `v1.2.0`, `v1.1.244`) across all `.md` files in `directives/` during every audit run.
- **Who triggers the update**: The agent or human that bumps `IDENTITY.md` is responsible for propagating the version string across all downstream directive files in the same commit/session.
- **Authoritative List**: As of `2026-07-13`, the following files carry an `IDENTITY.md` version reference and must be updated on each bump:
  - `GEMINI.md` — `Source of Truth` + `Identity Sync` fields
  - `TestWorkflow.md` — Compliance Heartbeat note
  - `neural_handoff.md` — Directive #2 reference
  - `compliance_check.md` — Directive #8 reference
  - `security_audit.md` — Directive #7 reference
  - `orchestrate.md` — Directive #3 reference
  - `emergency_shutdown.md` — Directive #7 reference
  - `incident_response.md` — Directive #6 reference
  - `deploy_to_prod.md` — Directive #7 reference
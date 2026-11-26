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
- [ ] Has every new term been added to `GLOSSARY.md` and `docs/wiki/Glossary.md`?
- [ ] Are all "Implementation Hooks" pointing to the correct files and methods?
- [ ] Have `[DEFINE:]` tags been applied to high-stakes sections of the manual?
- [ ] Does the `parity_guard.py` scan pass without warnings?
- [ ] Do all directives and routing tables preserve the Dual-Plane Sovereign Hierarchy (`Plane 1: Engine-Maintenance` vs. `Plane 2: SMB-Digital-Twin` and `FR-11` Permission Firewall) defined in `IDENTITY.md`?
- [ ] Have all directive files referencing `IDENTITY.md` by version been synchronized to `v1.1.464` (governed by `version.json` and `scripts/bump_version.cjs`)?

## 6. Directive Version Synchronization Policy
Any directive file that references `IDENTITY.md` by version string (e.g., `IDENTITY.md v1.1.464`) **must** be updated every time `IDENTITY.md` is versioned via `scripts/bump_version.cjs`.
- **Enforcement**: `parity_guard.py` and `scripts/bump_version.cjs --check` verify version and structural alignment across all target files during every audit run.
- **Who triggers the update**: The agent or human that bumps `IDENTITY.md` is responsible for propagating the version string across all downstream directive files in the same commit/session.
- **Authoritative List**: The following files carry an `IDENTITY.md` reference and must remain aligned on each bump:
  - `GEMINI.md` / `AGENTS.md` / `CLAUDE.md` / `directives/GEMINI.md` — Dual-Plane Sovereign Architecture + Directive #3, #6, #9 references
  - `TestWorkflow.md` — Dual-Plane verification lifecycle + Compliance Heartbeat note (Directive #8)
  - `neural_handoff.md` — Directive #2 reference + `[Execution-Plane]` schema
  - `compliance_check.md` — Directive #1 (`FR-11`) & Directive #8 references
  - `security_audit.md` — Directive #1 (`FR-11` / `NFR-1`) & Directive #7 references
  - `orchestrate.md` — Directive #2 (`[Execution-Plane]`) & Directive #3 references
  - `emergency_shutdown.md` — Directive #7 reference
  - `incident_response.md` — Directive #6 & `FR-11` references
  - `deploy_to_prod.md` — Directive #7 reference
  - `sme_discovery.md` — Directive #1 (`FR-11`) & Directive #9 references + Plane 2 SMB skills
  - `client_onboarding.md` — Directive #1 (`FR-11`) reference + Plane 2 SMB skills
  - `team_building.md` — Directive #1 (`FR-11`) capability provisioning reference
  - `campaign_launch.md` — `IDENTITY.md` brand compliance gate
  - `documentation_audit.md` — Dual-Plane Sovereign Hierarchy parity check
  - `system_architecture_review.md` — Dual-Plane (`FR-11` / `NFR-1`) isolation check

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[documentation_policy]` in audit logs.
>
> ### AI Assist Note
> Documentation Policy: Sovereign Semantic Standards
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

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

[//]: # (Metadata: [documentation_policy])

---
name: app-builder
description: Main application building orchestrator. Creates full-stack applications from natural language requests. Determines project type, selects tech stack, coordinates agents.
when_to_use: "When creating a new full-stack application from scratch, selecting tech stack, or scaffolding project structure. Use with /create workflow."
allowed-tools: Read, Write, Edit, Glob, Grep, Bash, Agent
version: "2.1.0"
priority: high
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / app-builder
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# App Builder — Application Building Orchestrator

> Analyzes user requests, determines technology stack, plans architecture, and coordinates agents.
> **Workflow Binding**: Used directly during the [`/create`](../../workflows/create.md) and [`/plan`](../../workflows/plan.md) workflows.

---

## 🎯 Progressive Disclosure & L3 Topic Index

Read **REQUIRED** core standards below; consult deep **L3 Topic Guides** on demand:

| Guide / Tool | Description | When to Read |
|---|---|---|
| [`references/project-detection.md`](./references/project-detection.md) | Keyword matrix, project type detection | Starting new project |
| [`references/tech-stack.md`](./references/tech-stack.md) | Standard stack recommendations & alternatives | Choosing technologies |
| [`references/agent-coordination.md`](./references/agent-coordination.md) | Agent pipeline, execution order | Coordinating multi-agent work |
| [`references/scaffolding.md`](./references/scaffolding.md) | Directory structure, core files | Creating project structure |
| [`references/feature-building.md`](./references/feature-building.md) | Feature analysis, error handling | Adding features to existing project |
| [`references/templates/templates_catalog.md`](./references/templates/templates_catalog.md) | Pre-configured application templates | Scaffolding new application |
| [`scripts/app_scaffolder.py`](./scripts/app_scaffolder.py) | Deterministic stack detector and scaffolding validator with `--test` suite | Automated stack detection or self-test verification |

---

## 🛡️ Socratic Gate: Scaffolding Pre-Conditions

Before scaffolding or generating full-stack structures:
1. **Target Runtime Confirmed**: Desktop app (Tauri), web SPA (Vite + React), or standalone backend service?
2. **Database & Storage Boundary**: Local SQLite (`sqlx`), vector memory (`LanceDB`), or external cloud?
3. **State Management**: Zustand store slices or localized component state?

---

## ⚠️ Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Operational Risk | Sovereign Remedy |
|---|---|---|
| Guessing Tech Stack | Incompatible libraries and broken toolchains | Run `app_scaffolder.py` or inspect existing package configs |
| Monolithic Feature Delivery | Giant unreviewable PRs and hallucinated bugs | Break work into discrete layers (schema -> API -> UI) |
| Hardcoded Credentials | Security breach via git commit leaks | Use `.env` with strict secrets masking rules |
| Premature Complex Frameworks | Over-engineering simple utilities | Choose the simplest stack that fulfills the specification |

---

## 🛠️ Verification & Execution Commands

```powershell
# Detect stack and validate directory
python .agent/skills/app-builder/scripts/app_scaffolder.py .

# Run self-test suite
python .agent/skills/app-builder/scripts/app_scaffolder.py --test
```
> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Documentation / ARCHITECTURE
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[ARCHITECTURE]`)

# Antigravity Kit Architecture

> Comprehensive AI Agent Capability Expansion Toolkit for Sovereign Swarm Intelligence

## 🧠 Core Intelligence, Dual-Plane Hierarchy & Routing

- **Dual-Plane Sovereign Hierarchy ([`directives/IDENTITY.md`](../directives/IDENTITY.md))**: Plane 1 is supported by the generic runner; Plane 2 is a planned business runtime. The current **`FR-11`** gate rejects only identities whose stored `department` exactly matches `Plane 2: SMB-Digital-Twin` (case-insensitive). This does not provide company-scoped authorization or isolation:
  1. **Plane 1 — Sovereign Codebase Lifecycle Engine (`Engine-Maintenance`)**: Recursively knows, operates, diagnoses, fixes, maintains, and improves the `AI-Tadpole-OS` codebase (`server-rs/`, `src/`, `execution/`, `ADG v2`, `parity_guard.py`, `Agent 99`).
  2. **Plane 2 — Company-Agnostic SMB AI Digital Twin (`SMB-Digital-Twin`, planned)**: Linked skills describe intended voice, ingestion, and PII procedures. Company-scoped workflows and workspace isolation remain pending per [`docs/specs/mock-smb-digital-twin.md`](../docs/specs/mock-smb-digital-twin.md). Until then, use synthetic inputs and stop at preview.
- **Intelligent Routing**: Located at [`intelligent-routing`](./skills/intelligent-routing/SKILL.md). This is the primary logic gate for autonomous agent selection, 4-dimensional prompt analysis (`Execution Plane`, `Domain`, `Complexity`, `Risk`), and task routing.
- **Skills Standard**: All modules conform to the open [Agent Skills Specification](https://agentskills.io) with progressive disclosure (L1 YAML metadata, L2 instructions, L3 references/assets). Comprehensive creation guide is available at [`README.md`](./skills/README.md).

---

## 📋 Overview

The Sovereign Swarm architecture is a modular, high-reliability system consisting of:

- **29 Specialist Agents** - Role-based AI personas with explicit skill bindings and domain boundaries
- **67 Skills** - Open-spec domain knowledge modules featuring deterministic execution scripts and rich reference assets
- **20 Workflows** - Deterministic slash command procedures governing everything from stress validation to production deployment

---

## 🏗️ Directory Structure

```plaintext
.agent/
├── ARCHITECTURE.md          # This file (Swarm Architecture & Component Registry)
├── agents/                  # 29 Specialist AI Agent Profiles
├── skills/                  # 67 Domain Knowledge Modules (agentskills.io spec)
│   └── doc.md               # Skills Creation & Progressive Disclosure Guide
├── workflows/               # 20 Slash Command Orchestration Procedures
├── memory/                  # Persistent Swarm & Agent Memory Bank
├── recipes/                 # Sovereign Workflow Recipes & Execution Plans
├── schemas/                 # Mission, Event, and Subagent Validation Schemas
├── globals.json             # Global Swarm Configuration
└── mcp_config.json          # Model Context Protocol Server Configuration

execution/                   # Deterministic Master Validation & Execution Scripts (Layer 3)
├── checklist.py             # Pre-commit & Core Verification Gate
├── verify_all.py            # Pre-deployment Full Verification Suite
├── parity_guard.py          # Architectural Parity & Documentation Guard
├── verify_doc_links.py      # Cross-Reference & Link Integrity Guard
└── verify_skill_specs.py    # Open-Spec Agent & Skill Conformance Linter
```

---

## 🤖 Agents (29)

Specialist AI personas for different domains. Each profile is defined in `.agent/agents/<agent-name>.md` with declared tool access, system directives, and bound skills.

| Agent | Focus & Architectural Mission | Bound Skills |
| ----- | ----------------------------- | ------------ |
| [`backend-specialist`](./agents/backend-specialist.md) | Backend architect specializing in high-availability, security-first infrastructure. Node.js, Python, API, DB, DevSecOps. | [`clean-code`](./skills/clean-code/SKILL.md), [`nodejs-best-practices`](./skills/nodejs-best-practices/SKILL.md), [`python-patterns`](./skills/python-patterns/SKILL.md), [`api-patterns`](./skills/api-patterns/SKILL.md), [`database-design`](./skills/database-design/SKILL.md), [`rust-pro`](./skills/rust-pro/SKILL.md), [`architecture`](./skills/architecture/SKILL.md), [`server-management`](./skills/server-management/SKILL.md) |
| [`code-archaeologist`](./agents/code-archaeologist.md) | Legacy code expert. Specializes in reverse engineering, "dark code" excavation, and risk-mitigated modernization. | [`code-review-graph`](./skills/code-review-graph/SKILL.md), [`simplify-code`](./skills/simplify-code/SKILL.md), [`verify-changes`](./skills/verify-changes/SKILL.md), [`architecture`](./skills/architecture/SKILL.md), [`clean-code`](./skills/clean-code/SKILL.md) |
| [`compliance-officer`](./agents/compliance-officer.md) | Legal & Compliance Architect. Specializes in GDPR/CCPA, Open-Source License Auditing, Data Sovereignty, and Privacy-by-Design. | [`vulnerability-scanner`](./skills/vulnerability-scanner/SKILL.md), [`pii-redaction`](./skills/pii-redaction/SKILL.md), [`architecture`](./skills/architecture/SKILL.md), [`web-design-guidelines`](./skills/web-design-guidelines/SKILL.md) |
| [`customer-backend-specialist`](./agents/customer-backend-specialist.md) | Customer-facing backend specialist focused on production safety, API compatibility, privacy, migrations, observability, idempotency, supportability, and rollout risk. | [`clean-code`](./skills/clean-code/SKILL.md), [`api-patterns`](./skills/api-patterns/SKILL.md), [`database-design`](./skills/database-design/SKILL.md), [`smb-data-ingestion`](./skills/smb-data-ingestion/SKILL.md), [`digital-twin-voice`](./skills/digital-twin-voice/SKILL.md), [`architecture`](./skills/architecture/SKILL.md), [`server-management`](./skills/server-management/SKILL.md) |
| [`database-architect`](./agents/database-architect.md) | Database architect specializing in schema design, query optimization, and zero-downtime migrations. | [`database-design`](./skills/database-design/SKILL.md), [`performance-profiling`](./skills/performance-profiling/SKILL.md), [`architecture`](./skills/architecture/SKILL.md) |
| [`debugger`](./agents/debugger.md) | Root Cause Analysis expert. Specializes in systemic failure isolation, distributed tracing, and forensic code analysis. | [`systematic-debugging`](./skills/systematic-debugging/SKILL.md), [`performance-profiling`](./skills/performance-profiling/SKILL.md), [`powershell-windows`](./skills/powershell-windows/SKILL.md), [`bash-linux`](./skills/bash-linux/SKILL.md) |
| [`devops-engineer`](./agents/devops-engineer.md) | Infrastructure and Operations architect. Specialist in CI/CD, IaC, Global Distribution, and Disaster Recovery. | [`deployment-procedures`](./skills/deployment-procedures/SKILL.md), [`server-management`](./skills/server-management/SKILL.md), [`vulnerability-scanner`](./skills/vulnerability-scanner/SKILL.md), [`bash-linux`](./skills/bash-linux/SKILL.md), [`powershell-windows`](./skills/powershell-windows/SKILL.md) |
| [`documentation-writer`](./agents/documentation-writer.md) | Technical Knowledge Architect. Specializes in API specifications, Architecture Decision Records (ADRs), and the systemic prevention of information decay. | [`documentation-templates`](./skills/documentation-templates/SKILL.md), [`architecture`](./skills/architecture/SKILL.md), [`api-patterns`](./skills/api-patterns/SKILL.md), [`i18n-localization`](./skills/i18n-localization/SKILL.md) |
| [`explorer-agent`](./agents/explorer-agent.md) | Codebase Discovery & Architectural Intelligence. Specializes in mapping critical paths, identifying technical debt, and detecting documentation-code drift. | [`clean-code`](./skills/clean-code/SKILL.md), [`systematic-debugging`](./skills/systematic-debugging/SKILL.md), [`explorer-scout`](./skills/explorer-scout/SKILL.md), [`code-review-graph`](./skills/code-review-graph/SKILL.md), [`architecture`](./skills/architecture/SKILL.md) |
| [`frontend-specialist`](./agents/frontend-specialist.md) | Senior Frontend Architect. Expert in React/Next.js, UI/UX Psychology, Accessibility, and Tailwind v4. Specializes in high-performance, bespoke interfaces. | [`clean-code`](./skills/clean-code/SKILL.md), [`nextjs-react-expert`](./skills/nextjs-react-expert/SKILL.md), [`frontend-design`](./skills/frontend-design/SKILL.md), [`tailwind-patterns`](./skills/tailwind-patterns/SKILL.md), [`web-design-guidelines`](./skills/web-design-guidelines/SKILL.md), [`performance-profiling`](./skills/performance-profiling/SKILL.md) |
| [`game-developer`](./agents/game-developer.md) | Game development expert. Unity, Godot, Phaser. Mechanics, Optimization, Multiplayer. | [`clean-code`](./skills/clean-code/SKILL.md), [`game-development`](./skills/game-development/SKILL.md) |
| [`growth-engineer`](./agents/growth-engineer.md) | Data Scientist & Growth Architect. Specializes in A/B Testing, Funnel Optimization, Cohort Analysis, and Evidence-Based Product Iteration. | [`seo-fundamentals`](./skills/seo-fundamentals/SKILL.md), [`frontend-design`](./skills/frontend-design/SKILL.md), [`nextjs-react-expert`](./skills/nextjs-react-expert/SKILL.md), [`web-design-guidelines`](./skills/web-design-guidelines/SKILL.md) |
| [`mobile-developer`](./agents/mobile-developer.md) | Mobile expert. Jetpack Compose, SwiftUI, React Native, Flutter. iOS/Android nuances. | [`clean-code`](./skills/clean-code/SKILL.md), [`mobile-design-android`](./skills/mobile-design-android/SKILL.md), [`mobile-design-ios`](./skills/mobile-design-ios/SKILL.md) |
| [`nexus-engineer`](./agents/nexus-engineer.md) | Fused Master System Architect, Principal QA, and Chief Security Auditor. Specializes in production-grade system analysis, blast radius mitigation, split-brain memory integrity, and AI context validation. | [`clean-code`](./skills/clean-code/SKILL.md), [`code-review`](./skills/code-review/SKILL.md), [`code-review-checklist`](./skills/code-review-checklist/SKILL.md), [`code-review-graph`](./skills/code-review-graph/SKILL.md), [`architecture`](./skills/architecture/SKILL.md), [`improve-codebase-architecture`](./skills/improve-codebase-architecture/SKILL.md), [`vulnerability-scanner`](./skills/vulnerability-scanner/SKILL.md), [`testing-patterns`](./skills/testing-patterns/SKILL.md), [`verify-changes`](./skills/verify-changes/SKILL.md), [`powershell-windows`](./skills/powershell-windows/SKILL.md), [`rust-pro`](./skills/rust-pro/SKILL.md), [`nextjs-react-expert`](./skills/nextjs-react-expert/SKILL.md) |
| [`orchestrator`](./agents/orchestrator.md) | Master Agent for Multi-Agent Coordination. Decomposes complex objectives into executable pipelines, invokes specialists, and synthesizes disparate outputs into a unified Truth. | [`parallel-agents`](./skills/parallel-agents/SKILL.md), [`coordinator-mode`](./skills/coordinator-mode/SKILL.md), [`architecture`](./skills/architecture/SKILL.md), [`wayfinder`](./skills/wayfinder/SKILL.md), [`handoff`](./skills/handoff/SKILL.md), [`lint-and-validate`](./skills/lint-and-validate/SKILL.md) |
| [`penetration-tester`](./agents/penetration-tester.md) | Offensive Security Engineer. Specializes in vulnerability research, red-team tactics, and the exploitation of logical flaws in APIs and Infrastructure-as-Code. | [`red-team-tactics`](./skills/red-team-tactics/SKILL.md), [`vulnerability-scanner`](./skills/vulnerability-scanner/SKILL.md), [`api-patterns`](./skills/api-patterns/SKILL.md) |
| [`performance-optimizer`](./agents/performance-optimizer.md) | Full-Stack Performance Architect. Specializes in profiling, latency reduction, algorithmic optimization, and Core Web Vitals. Operates on the principle of "Evidence over Intuition." | [`performance-profiling`](./skills/performance-profiling/SKILL.md), [`seo-fundamentals`](./skills/seo-fundamentals/SKILL.md), [`architecture`](./skills/architecture/SKILL.md) |
| [`product-manager`](./agents/product-manager.md) | Product Requirements Engineer. Specializes in User Experience (UX) strategy, Requirement Decomposition, and Value-Driven Prioritization. | [`plan-writing`](./skills/plan-writing/SKILL.md), [`brainstorming`](./skills/brainstorming/SKILL.md), [`web-design-guidelines`](./skills/web-design-guidelines/SKILL.md) |
| [`product-owner`](./agents/product-owner.md) | Strategic Product Governor. Specializes in Value Stream Mapping, ROI Analysis, Roadmap Orchestration, and Strategic Trade-offs. | [`plan-writing`](./skills/plan-writing/SKILL.md), [`brainstorming`](./skills/brainstorming/SKILL.md), [`architecture`](./skills/architecture/SKILL.md) |
| [`project-planner`](./agents/project-planner.md) | Technical Architect & Strategist. Specializes in task decomposition, system design, and the creation of executable blueprints. Converts "What" (PRD) into "How" (Atomic Tasks). | [`plan-writing`](./skills/plan-writing/SKILL.md), [`architecture`](./skills/architecture/SKILL.md) |
| [`qa-automation-engineer`](./agents/qa-automation-engineer.md) | Quality Assurance & Automation Architect. Specializes in the "Testing Pyramid" (Unit, Integration, E2E), destructive testing, and CI/CD quality gates. The final arbiter of "Done." | [`webapp-testing`](./skills/webapp-testing/SKILL.md), [`testing-patterns`](./skills/testing-patterns/SKILL.md), [`red-team-tactics`](./skills/red-team-tactics/SKILL.md) |
| [`release-manager`](./agents/release-manager.md) | Delivery & Versioning Architect. Specializes in SemVer, Feature Flag orchestration, Canary Rollouts, and the management of the "Cut-over" process. | [`deployment-procedures`](./skills/deployment-procedures/SKILL.md), [`verify-changes`](./skills/verify-changes/SKILL.md) |
| [`security-auditor`](./agents/security-auditor.md) | Defensive Security Architect & Auditor. Specializes in Zero Trust architecture, compliance (OWASP, NIST), systemic hardening, and vulnerability management. | [`vulnerability-scanner`](./skills/vulnerability-scanner/SKILL.md), [`red-team-tactics`](./skills/red-team-tactics/SKILL.md), [`architecture`](./skills/architecture/SKILL.md) |
| [`seo-specialist`](./agents/seo-specialist.md) | Visibility Architect. Specializes in Search Engine Optimization (SEO) and Generative/Answer Engine Optimization (GEO/AEO). Expert in Semantic HTML, Schema.org, and Entity-based content strategy. | [`seo-fundamentals`](./skills/seo-fundamentals/SKILL.md), [`geo-fundamentals`](./skills/geo-fundamentals/SKILL.md) |
| [`sre-architect`](./agents/sre-architect.md) | Site Reliability Engineer. Specializes in SLO/SLI management, incident response, observability, and high-availability system hardening. | [`server-management`](./skills/server-management/SKILL.md), [`performance-profiling`](./skills/performance-profiling/SKILL.md), [`red-team-tactics`](./skills/red-team-tactics/SKILL.md) |
| [`tadpole-backend-specialist`](./agents/tadpole-backend-specialist.md) | Tadpole OS backend specialist for Rust, Axum, Tokio, sqlx, SQLite, AppState, agent registry, runner lifecycle, WebSocket events, and backend contract integrity. | [`clean-code`](./skills/clean-code/SKILL.md), [`rust-pro`](./skills/rust-pro/SKILL.md), [`api-patterns`](./skills/api-patterns/SKILL.md), [`database-design`](./skills/database-design/SKILL.md), [`architecture`](./skills/architecture/SKILL.md), [`server-management`](./skills/server-management/SKILL.md) |
| [`tadpole-frontend-specialist`](./agents/tadpole-frontend-specialist.md) | Tadpole OS frontend specialist for React, Vite, TypeScript, Zustand, Tailwind v4, WebSockets integration, Tauri bridge, and frontend-backend contracts. | [`clean-code`](./skills/clean-code/SKILL.md), [`nextjs-react-expert`](./skills/nextjs-react-expert/SKILL.md), [`frontend-design`](./skills/frontend-design/SKILL.md), [`tailwind-patterns`](./skills/tailwind-patterns/SKILL.md), [`web-design-guidelines`](./skills/web-design-guidelines/SKILL.md), [`performance-profiling`](./skills/performance-profiling/SKILL.md) |
| [`test-engineer`](./agents/test-engineer.md) | TDD & Logic Verification Expert. Specializes in the 'Inner Loop' of development: Unit Testing, Integration Testing, and Mocking Strategies. | [`tdd-workflow`](./skills/tdd-workflow/SKILL.md), [`testing-patterns`](./skills/testing-patterns/SKILL.md) |
| [`ux-designer`](./agents/ux-designer.md) | Product & UX Designer. Specializes in User Psychology, Interaction Design, User Journey Mapping, and Design Systems. | [`web-design-guidelines`](./skills/web-design-guidelines/SKILL.md), [`frontend-design`](./skills/frontend-design/SKILL.md), [`tailwind-patterns`](./skills/tailwind-patterns/SKILL.md) |

---

## 🧩 Skills (67)

Modular knowledge domains adhering to the [Agent Skills Specification](https://agentskills.io). Loaded on-demand via progressive disclosure based on prompt intent and task routing.
### Frontend & UI (6)

| Skill | Description |
| ----- | ----------- |
| [`nextjs-react-expert`](./skills/nextjs-react-expert/SKILL.md) | React, Next.js, and Vite SPA performance optimization. Covers waterfall elimination, bundle size reduction, React 19 hooks, and client-side desktop performance. |
| [`frontend-design`](./skills/frontend-design/SKILL.md) | Design thinking and decision-making for web UI. Use when designing components, layouts, color schemes, typography, or creating aesthetic interfaces. Teaches principles, not fixed values. |
| [`tailwind-patterns`](./skills/tailwind-patterns/SKILL.md) | Tailwind CSS v4 principles. CSS-first configuration, container queries, modern patterns, design token architecture. |
| [`web-design-guidelines`](./skills/web-design-guidelines/SKILL.md) | Review UI code for Web Interface Guidelines compliance. Focus outlines, touch targets, accessibility (WCAG AA), typography, and interactive affordances. |
| [`zustand-patterns`](./skills/zustand-patterns/SKILL.md) | Zustand state store architecture, slice patterns, selector memoization with useShallow, and outside-component actions. |
| [`prototype`](./skills/prototype/SKILL.md) | Builds minimal, low-fidelity executable spikes or stub components to resolve 'how should it look/behave' questions before production coding. |

### Backend & Systems (7)

| Skill | Description |
| ----- | ----------- |
| [`rust-pro`](./skills/rust-pro/SKILL.md) | Master Rust 1.75+ with modern async patterns, advanced type system features, and production-ready systems programming. Expert in Tokio, Axum, and zero-cost abstractions. |
| [`tauri-pro`](./skills/tauri-pro/SKILL.md) | Master Tauri v1/v2 desktop development with Rust commands, IPC bridge, security capabilities, window state, and event bus. |
| [`api-patterns`](./skills/api-patterns/SKILL.md) | API design principles and decision-making. REST vs GraphQL vs tRPC selection, response formats, versioning, pagination. |
| [`nodejs-best-practices`](./skills/nodejs-best-practices/SKILL.md) | Node.js development principles and decision-making. Framework selection, async patterns, security, and architecture. Teaches thinking, not copying. |
| [`python-patterns`](./skills/python-patterns/SKILL.md) | Python development principles and decision-making. Framework selection, async patterns, type hints, and project structure. |
| [`server-management`](./skills/server-management/SKILL.md) | Server management principles and decision-making. Process supervision, systemd units, PM2 clustering, health monitoring, and scaling strategies. |
| [`deployment-procedures`](./skills/deployment-procedures/SKILL.md) | Production deployment principles and decision-making. Safe deployment workflows, rollback strategies, and verification. Teaches thinking, not scripts. |

### Database & Vector Memory (4)

| Skill | Description |
| ----- | ----------- |
| [`database-design`](./skills/database-design/SKILL.md) | Database design principles, schema modeling, migration strategies, indexing, and ORM selection. Covers SQLite/sqlx, PostgreSQL/Neon, Turso, LanceDB vector memory, Prisma, and Drizzle. |
| [`vector-memory-lancedb`](./skills/vector-memory-lancedb/SKILL.md) | Dense vector memory, LanceDB table schemas, embedding pipelines, cosine similarity deduplication, and semantic RAG retrieval. |
| [`domain-modeling`](./skills/domain-modeling/SKILL.md) | Actively builds and sharpens the project's domain model, ubiquitous glossary (CONTEXT.md / docs/GLOSSARY.md), and architectural decision records (docs/adr/). |
| [`smb-data-ingestion`](./skills/smb-data-ingestion/SKILL.md) | Parses and validates messy SMB data exports (CSV, Excel, Quickbooks PDFs, legacy SQL) into clean JSON-Schema & SQLite state for AI Digital Twins. |

### Mobile Development (2)

| Skill | Description |
| ----- | ----------- |
| [`mobile-design-android`](./skills/mobile-design-android/SKILL.md) | Mobile-first Android design thinking, Jetpack Compose, Material 3 (M3), Compose Navigation 3, Compose Material 3 Adaptive layouts, StateFlow / ViewModel state management, and Android platform conventions. Synchronized with official Google Developer documentation. |
| [`mobile-design-ios`](./skills/mobile-design-ios/SKILL.md) | Mobile-first iOS design thinking, SwiftUI, UIKit, Apple Human Interface Guidelines (HIG), SF Symbols, NavigationStack/SplitView, @Observable macro, SwiftData/Core Data, and iOS platform conventions. |

### Security, Governance & Red Teaming (5)

| Skill | Description |
| ----- | ----------- |
| [`vulnerability-scanner`](./skills/vulnerability-scanner/SKILL.md) | Advanced vulnerability analysis principles. OWASP Agentic Top 10 (2026), MITRE ATLAS, Supply Chain Security, secret scanning, and attack surface mapping. |
| [`red-team-tactics`](./skills/red-team-tactics/SKILL.md) | Red team tactics principles based on MITRE ATT&CK v19.2 and MITRE ATLAS v2026.07. Stealth, defense impairment, and agentic swarm attack phases. |
| [`pii-redaction`](./skills/pii-redaction/SKILL.md) | Zero-Trust PII sanitizer that redacts customer credit cards, phone numbers, SSNs, emails, and API keys before data is sent to cloud LLM providers. |
| [`a2a-protocol`](./skills/a2a-protocol/SKILL.md) | Agent-to-Agent (A2A) economic protocol, Web3 vault settlement, capability key rotation, and cryptographic oversight signatures. |
| [`adg-governance`](./skills/adg-governance/SKILL.md) | Active Documentation Guard (ADG v2) protocol, machine-maintained witness manifests, AST symbol blast radius analysis, and parity guard compliance. |

### Testing, QA & Verification (8)

| Skill | Description |
| ----- | ----------- |
| [`testing-patterns`](./skills/testing-patterns/SKILL.md) | Testing patterns and principles. Unit, integration, mocking strategies, Arrange-Act-Assert (AAA), and multi-ecosystem test runners. |
| [`webapp-testing`](./skills/webapp-testing/SKILL.md) | Web application testing principles. E2E, Playwright, deep audit strategies. |
| [`tdd-workflow`](./skills/tdd-workflow/SKILL.md) | Test-Driven Development workflow principles. RED-GREEN-REFACTOR cycle, writing failing tests first, and enforcing behavioral verification. |
| [`verify-changes`](./skills/verify-changes/SKILL.md) | Prove code works by running it, not just checking it exists. Verification through execution rather than inspection. Use after writing or modifying code to ensure it actually functions correctly. |
| [`code-review`](./skills/code-review/SKILL.md) | Two-axis parallel code review comparing HEAD against a fixed point. Axis 1 (Standards) audits repo conventions and Fowler smells; Axis 2 (Spec) verifies line-by-line requirements coverage. |
| [`code-review-checklist`](./skills/code-review-checklist/SKILL.md) | Systematic code review audit engine and master rubric across logic, security hardening, performance, clean-code adherence, and test coverage. |
| [`code-review-graph`](./skills/code-review-graph/SKILL.md) | Token-efficient code review using Tree-sitter AST graphs and MCP. Reduces AI assistant token usage by up to 49x by computing blast radius instead of reading entire folders. |
| [`lint-and-validate`](./skills/lint-and-validate/SKILL.md) | Automatic quality control, linting, and static analysis procedures. Use after every code modification to ensure syntax correctness and project standards. Triggers onKeywords: lint, format, check, validate, types, static analysis. |

### Debugging & Diagnostics (2)

| Skill | Description |
| ----- | ----------- |
| [`systematic-debugging`](./skills/systematic-debugging/SKILL.md) | 4-phase systematic debugging methodology with root cause analysis and evidence-based verification. Use when debugging complex issues. |
| [`performance-profiling`](./skills/performance-profiling/SKILL.md) | Performance profiling principles. Measurement, analysis, and optimization techniques. Core Web Vitals, Lighthouse audits, flame graphs, and bundle inspection. |

### Architecture, Planning & Reasoning (9)

| Skill | Description |
| ----- | ----------- |
| [`architecture`](./skills/architecture/SKILL.md) | Architectural decision-making framework. Requirements analysis, trade-off evaluation, ADR documentation. Use when making architecture decisions or analyzing system design. |
| [`improve-codebase-architecture`](./skills/improve-codebase-architecture/SKILL.md) | Scans git commit hot spots for shallow modules, generates visual HTML reports with Mermaid diagrams, and drills into refactoring candidates. |
| [`app-builder`](./skills/app-builder/SKILL.md) | Main application building orchestrator. Creates full-stack applications from natural language requests. Determines project type, selects tech stack, coordinates agents. |
| [`plan-writing`](./skills/plan-writing/SKILL.md) | Structured task planning with clear breakdowns, dependencies, and verification criteria. Use when implementing features, refactoring, or any multi-step work. |
| [`brainstorming`](./skills/brainstorming/SKILL.md) | Socratic questioning protocol and communication framework. Mandatory operational gateway for complex requests, ambiguous requirements, and architectural trade-off alignment. |
| [`aletheia-reasoning`](./skills/aletheia-reasoning/SKILL.md) | Structured iterative reasoning framework for deep problem solving, featuring generator, verifier, reviser, and final proof stages. |
| [`to-spec`](./skills/to-spec/SKILL.md) | Converts informal conversation context or brainstorms into formal, published specification files under docs/specs/. |
| [`wayfinder`](./skills/wayfinder/SKILL.md) | Plan large multi-session initiatives wrapped in fog as a map of linked decision tickets, resolving them sequentially without context degradation. |
| [`world-model-synthesis`](./skills/world-model-synthesis/SKILL.md) | Synthesizes programmatic executable world models (.tmp/world_model.py) and executes local graph search (BFS/A*) for deterministic agent task planning. |

### Multi-Agent Orchestration & Swarm (6)

| Skill | Description |
| ----- | ----------- |
| [`intelligent-routing`](./skills/intelligent-routing/SKILL.md) | Automatic agent selection and intelligent task routing. Analyzes user requests, classifies intent and complexity, and automatically selects the best specialist agent(s) and skills without requiring explicit user mentions. |
| [`coordinator-mode`](./skills/coordinator-mode/SKILL.md) | Advanced multi-agent orchestration with parallel workers, synthesis protocols, and coordinator lifecycle. Use when complex tasks require multiple agents working in parallel with intelligent result synthesis. |
| [`parallel-agents`](./skills/parallel-agents/SKILL.md) | Multi-agent orchestration patterns. Use when multiple independent tasks can run with different domain expertise or when comprehensive analysis requires multiple perspectives. |
| [`behavioral-modes`](./skills/behavioral-modes/SKILL.md) | Adaptive AI operational modes (BRAINSTORM, ORCHESTRATE, ALETHEIA, IMPLEMENT, DEBUG, REVIEW, TEACH, SHIP) and PEC handover protocols to eliminate mode bleed, premature execution, and conversational drift. |
| [`handoff`](./skills/handoff/SKILL.md) | Compacts current session state into a clean, deterministic handoff document for incoming agents or multi-session state transfers. Prevents context drift and cold-start latency. |
| [`mission-analyst`](./skills/mission-analyst/SKILL.md) | Agent 99 persona for automated post-mission debriefing. Extracts architectural wisdom and updates LONG_TERM_MEMORY.md. Triggers on: debrief, post-mission, extract wisdom, synthesize learnings. |

### Memory & Context Management (4)

| Skill | Description |
| ----- | ----------- |
| [`memory-system`](./skills/memory-system/SKILL.md) | Persistent cross-session memory management. Enables agents to remember user preferences, project conventions, and past decisions across different sessions using a structured MEMORY.md index and topic files. |
| [`context-compression`](./skills/context-compression/SKILL.md) | Manage and compress conversation context in long sessions. Detect when context is growing large, summarize completed work phases, archive old findings while preserving key decisions. Prevents context degradation. |
| [`digital-twin-voice`](./skills/digital-twin-voice/SKILL.md) | Extracts SMB founder/brand tone, formality, and communication phrasing to calibrate AI Digital Twin persona artifacts. |
| [`gemma4-local`](./skills/gemma4-local/SKILL.md) | Provides core guidelines and prompt formatting patterns for interacting with local gemma4 models. Triggers when gemma4 is used as the local LLM backend. |

### Code Engineering & Automation (4)

| Skill | Description |
| ----- | ----------- |
| [`clean-code`](./skills/clean-code/SKILL.md) | Pragmatic coding standards: concise, direct, zero over-engineering, guard clauses, cognitive load minimization, and language idioms (Rust, TS, Python). |
| [`simplify-code`](./skills/simplify-code/SKILL.md) | Reduce complexity of over-engineered code. Identify unnecessary abstractions, remove dead code, flatten deep nesting, and simplify logic while preserving behavior. |
| [`skillify`](./skills/skillify/SKILL.md) | Auto-create new skills from repetitive workflows. When you notice yourself doing the same multi-step process repeatedly, extract it into a reusable SKILL.md that any agent can use. |
| [`batch-operations`](./skills/batch-operations/SKILL.md) | Apply operations across multiple files simultaneously. Pattern-based bulk modifications, search-and-replace across codebases, consistent changes to many files at once. |

### CLI & Platform Tools (3)

| Skill | Description |
| ----- | ----------- |
| [`bash-linux`](./skills/bash-linux/SKILL.md) | Bash/Linux terminal patterns. Critical commands, piping, error handling, scripting. Use when working on macOS or Linux systems. |
| [`powershell-windows`](./skills/powershell-windows/SKILL.md) | PowerShell Windows patterns. Critical pitfalls, operator syntax, error handling, JSON serialization, and cmdlets. |
| [`mcp-builder`](./skills/mcp-builder/SKILL.md) | MCP (Model Context Protocol) server building principles. Tool design, resource patterns, JSON-RPC 2.0 schemas, and transport standards. |

### Documentation, SEO & Growth (6)

| Skill | Description |
| ----- | ----------- |
| [`documentation-templates`](./skills/documentation-templates/SKILL.md) | Standardized documentation protocols for humans and AI. Manages READMEs, API specs, ADRs, and AI-native context maps. |
| [`seo-fundamentals`](./skills/seo-fundamentals/SKILL.md) | SEO fundamentals, E-E-A-T, Core Web Vitals, metadata schemas, OpenGraph tags, and Google algorithm principles. |
| [`geo-fundamentals`](./skills/geo-fundamentals/SKILL.md) | Generative Engine Optimization execution protocol. Audits and injects structured schemas, statistical citation magnets, and E-E-A-T entity markers. |
| [`i18n-localization`](./skills/i18n-localization/SKILL.md) | Internationalization (i18n) and localization (L10n) patterns. Detecting hardcoded strings, managing translations, locale files, RTL support, and Intl formatting. |
| [`explorer-scout`](./skills/explorer-scout/SKILL.md) | Scouting and architectural analysis for GitHub repositories. Extracts manifest dependencies, licensing, and structural patterns. |
| [`research`](./skills/research/SKILL.md) | Executes isolated research and investigation subagents on throwaway git branches to prevent cluttering main working state. |

### Specialized & Game Development (1)

| Skill | Description |
| ----- | ----------- |
| [`game-development`](./skills/game-development/SKILL.md) | Game development orchestrator. Routes to platform-specific and genre-specific game engine reference guides. |

---

## 🔄 Workflows (20)

Deterministic slash command procedures. Invoke with `/command`. Each workflow defines strict preconditions, multi-agent routing, tool calling gates, and invariant verification pipelines.

| Command | Description & Operational Focus |
| ------- | ------------------------------- |
| [`/adversary`](./workflows/adversary.md) | Sovereign Stress Validation. An adversarial workflow designed to uncover fragility, logic gaps, and "Edge-Case Collapse" through simulated attacks and chaos engineering. |
| [`/anneal`](./workflows/anneal.md) | Annealing is the process of hardening the Sovereign OS instruction-set by iteratively resolving "faults." In this context, a fault is defined as a discrepancy where the system's output deviates from the intended sovereign logic. |
| [`/audit`](./workflows/audit.md) | Sovereign Security & Compliance Audit. Executes deep system scans and verifies documentation parity. |
| [`/brainstorm`](./workflows/brainstorm.md) | Structured brainstorming for projects and features. Explores multiple divergent options and evaluates them against Sovereign benchmarks before implementation. |
| [`/codify`](./workflows/codify.md) | Knowledge Heritage Synthesis. Transforms successful ephemeral patterns and "intelligence wins" into permanent Sovereign Laws and Core Directives. |
| [`/create`](./workflows/create.md) | Orchestrates the end-to-end creation of new applications. Integrates discovery, brainstorming, blueprinting, and multi-agent execution. |
| [`/debug`](./workflows/debug.md) | Initiates the Sovereign Root Cause Analysis (RCA) protocol. Systematically executes a multi-phase forensic investigation, gathering telemetry, isolating fault vectors, and determining the Point of Divergence between code and directive. |
| [`/deploy`](./workflows/deploy.md) | Production State Transition. Manages the movement of audited, hardened, and verified code into live environments via a strict integrity pipeline. |
| [`/enhance`](./workflows/enhance.md) | Controlled evolutionary growth of existing applications. Ensures new features are integrated without introducing architectural drift or regressions. |
| [`/github_scout`](./workflows/github_scout.md) | Project Intelligence Research. Performs deep architectural extraction of external repositories to identify innovation vectors and implementation patterns for the Sovereign Registry. |
| [`/migrate`](./workflows/migrate.md) | Sovereign State Transition. Manages the deterministic evolution of the Registry (Database) schema to ensure zero data loss and zero structural drift. |
| [`/onboard`](./workflows/onboard.md) | Agent Context Synchronization. Rapidly aligns new or returning AI agents with the current Sovereign state, eliminating "Cold-Start" drift and architectural hallucinations. |
| [`/orchestrate`](./workflows/orchestrate.md) | Sovereign Governor for multi-agent orchestration. Coordinates specialized agents and systemic workflows to execute complex architectural transitions with zero drift. |
| [`/plan`](./workflows/plan.md) | Sovereign Blueprinting. Coordinates the project-planner agent to translate brainstormed ideas into a deterministic architectural contract. No code is written during this phase |
| [`/preview`](./workflows/preview.md) | Sovereign Sandbox Management. Handles the initialization, monitoring, and termination of the local simulation environment to verify architectural integrity before production deployment. |
| [`/refactor`](./workflows/refactor.md) | Structural Hardening & Architectural Refinement. Focuses on reducing system entropy and decoupling the 4-Pillars without altering external behavior. |
| [`/report`](./workflows/report.md) | Sovereign Intelligence Synthesis. Transforms raw swarm telemetry and architectural drift data into high-fidelity dossiers to drive the system's evolutionary loop. |
| [`/status`](./workflows/status.md) | Sovereign Integrity Dashboard. Provides a real-time snapshot of the system's operational health and architectural alignment across the 4-Pillars |
| [`/test`](./workflows/test.md) | Sovereign Verification Gate. A deterministic pipeline to prove architectural integrity and behavioral parity before any state transition to production. |
| [`/ui-ux-pro-max`](./workflows/ui-ux-pro-max.md) | Sovereign Gateway Visual Protocol. An AI-driven architectural framework for generating deterministic design systems and high-fidelity UI/UX implementations. |

---

## 🎯 Skill Loading Protocol

The system dynamically resolves and discloses skills using progressive levels:

```plaintext
User Intent / Slash Command
          ↓
Keyword & Semantic Classification (.agent/skills/intelligent-routing/scripts/route_dispatcher.py)
          ↓
Level 1: YAML Frontmatter (Name & Description loaded into routing prompt)
          ↓
Level 2: Procedural Body (SKILL.md instructions injected into active context)
          ↓
Level 3: Deep Execution (Deterministic scripts in scripts/, heavy schemas in references/)
```

### Skill Structure Standard

```plaintext
skill-name/
├── SKILL.md           # (Required) Level 1 metadata & Level 2 actionable instructions
├── scripts/           # (Optional) Deterministic Python/Bash/JS execution scripts
├── references/        # (Optional) Domain references, schemas, checklists, and templates
└── assets/            # (Optional) Visual assets, diagrams, and figures
```

### Enhanced Skills (Extensive Reference & Script Libraries)

| Skill | Resource Files | Coverage & Assets |
| ----- | -------------- | ----------------- |
| [`mobile-design-android`](./skills/mobile-design-android/SKILL.md) | 13 | Jetpack Compose M3, Adaptive layouts, touch psychology, mobile testing |
| [`api-patterns`](./skills/api-patterns/SKILL.md) | 11 | REST, GraphQL, tRPC standards, pagination, API versioning & error schemas |
| [`game-development`](./skills/game-development/SKILL.md) | 11 | Platform & genre engine guides, loop patterns, physics & math references |
| [`nextjs-react-expert`](./skills/nextjs-react-expert/SKILL.md) | 11 | React 19 rules, Vite SPA optimization, rule converter, performance auditing |
| [`frontend-design`](./skills/frontend-design/SKILL.md) | 9 | Accessibility audit, UX review, design tokens, micro-interactions |
| [`app-builder`](./skills/app-builder/SKILL.md) | 7 | Full-stack scaffolding, tech stack matrix, starter kit automation |
| [`database-design`](./skills/database-design/SKILL.md) | 7 | SQLite/sqlx schemas, indexing rubrics, migrations, query optimization |
| [`architecture`](./skills/architecture/SKILL.md) | 6 | ADR templates, blast radius mitigation, trade-off evaluation matrices |
| [`mobile-design-ios`](./skills/mobile-design-ios/SKILL.md) | 5 | SwiftUI, Apple HIG guidelines, navigation stacks, iOS validator |

---

## 🛠️ Master Validation & Execution Scripts

Deterministic Layer 3 Python scripts in `execution/` enforce sovereign system integrity across code, documentation, and agent specifications.

### Master Scripts Matrix

| Script | Purpose | When to Use |
| ------ | ------- | ----------- |
| [`execution/checklist.py`](../execution/checklist.py) | Priority-based validation (P0: Security & Lint; P1: Docs & Schema; P2: Tests & UX) | Development cycles, pre-commit gate |
| [`execution/verify_all.py`](../execution/verify_all.py) | Full verification release suite (Checklist + Lighthouse CWV, Playwright E2E, Bundle, i18n) | Pre-deployment, release builds |
| [`execution/parity_guard.py`](../execution/parity_guard.py) | Architectural documentation parity, tag synchronization, and env var verification | All commits, CI pipeline gate |
| [`execution/verify_skill_specs.py`](../execution/verify_skill_specs.py) | Open-spec validation for all 67 skills, 29 subagents, and 20 workflows | Skill authoring, agent updates |
| [`execution/verify_doc_links.py`](../execution/verify_doc_links.py) | Complete cross-reference and markdown link integrity guard | Documentation updates |

### Verification Usage

```bash
# 1. Quick validation during development
python execution/checklist.py .

# 2. Architectural documentation parity check
python execution/parity_guard.py .

# 3. Agent and skill specification conformance audit
python execution/verify_skill_specs.py

# 4. Cross-reference link integrity audit
python execution/verify_doc_links.py

# 5. Full release verification suite before production deployment
python execution/verify_all.py . --url http://localhost:8000
```

---

## 📊 Statistics

| Metric | Value | Reference / Verification Command |
| ------ | ----- | -------------------------------- |
| **Total Specialist Agents** | 29 | `ls .agent/agents/*.md` (`verify_skill_specs.py`) |
| **Total Domain Skills** | 67 | `ls .agent/skills/` (`verify_skill_specs.py`) |
| **Total Slash Workflows** | 20 | `ls .agent/workflows/*.md` (`verify_skill_specs.py`) |
| **Skill-Level Automation Scripts** | 71 | `find .agent/skills -path '*/scripts/*' -type f` |
| **Master Execution Scripts** | 5 | [`execution/`](../execution) |
| **Specification Conformance** | 100% (116/116 passed) | `python execution/verify_skill_specs.py` |
| **Documentation Link Integrity** | 100% healthy | `python execution/verify_doc_links.py` |

---

## 🔗 Quick Reference & Routing Map

| Operational Need | Primary Specialist Agent | Bound Skills |
| ---------------- | ------------------------ | ------------ |
| **Full-Stack Desktop App** | [`tadpole-frontend-specialist`](./agents/tadpole-frontend-specialist.md) / [`tadpole-backend-specialist`](./agents/tadpole-backend-specialist.md) | [`tauri-pro`](./skills/tauri-pro/SKILL.md), [`nextjs-react-expert`](./skills/nextjs-react-expert/SKILL.md), [`rust-pro`](./skills/rust-pro/SKILL.md) |
| **Rust Axum & Tauri IPC** | [`tadpole-backend-specialist`](./agents/tadpole-backend-specialist.md) | [`rust-pro`](./skills/rust-pro/SKILL.md), [`tauri-pro`](./skills/tauri-pro/SKILL.md), [`api-patterns`](./skills/api-patterns/SKILL.md) |
| **React UI, Zustand & Tailwind** | [`tadpole-frontend-specialist`](./agents/tadpole-frontend-specialist.md) | [`frontend-design`](./skills/frontend-design/SKILL.md), [`tailwind-patterns`](./skills/tailwind-patterns/SKILL.md), [`zustand-patterns`](./skills/zustand-patterns/SKILL.md) |
| **System Audit & Blast Radius** | [`nexus-engineer`](./agents/nexus-engineer.md) | [`architecture`](./skills/architecture/SKILL.md), [`improve-codebase-architecture`](./skills/improve-codebase-architecture/SKILL.md), [`code-review-graph`](./skills/code-review-graph/SKILL.md) |
| **Multi-Agent Coordination** | [`orchestrator`](./agents/orchestrator.md) | [`coordinator-mode`](./skills/coordinator-mode/SKILL.md), [`parallel-agents`](./skills/parallel-agents/SKILL.md), [`handoff`](./skills/handoff/SKILL.md) |
| **Database & Vector Memory** | [`database-architect`](./agents/database-architect.md) | [`database-design`](./skills/database-design/SKILL.md), [`vector-memory-lancedb`](./skills/vector-memory-lancedb/SKILL.md), [`domain-modeling`](./skills/domain-modeling/SKILL.md) |
| **Defensive Security & Auth** | [`security-auditor`](./agents/security-auditor.md) | [`vulnerability-scanner`](./skills/vulnerability-scanner/SKILL.md), [`pii-redaction`](./skills/pii-redaction/SKILL.md) |
| **Offensive Security & Red Team** | [`penetration-tester`](./agents/penetration-tester.md) | [`red-team-tactics`](./skills/red-team-tactics/SKILL.md), [`vulnerability-scanner`](./skills/vulnerability-scanner/SKILL.md) |
| **Testing & Invariant Verification** | [`test-engineer`](./agents/test-engineer.md) / [`qa-automation-engineer`](./agents/qa-automation-engineer.md) | [`tdd-workflow`](./skills/tdd-workflow/SKILL.md), [`testing-patterns`](./skills/testing-patterns/SKILL.md), [`webapp-testing`](./skills/webapp-testing/SKILL.md) |
| **Root Cause Debugging** | [`debugger`](./agents/debugger.md) | [`systematic-debugging`](./skills/systematic-debugging/SKILL.md), [`performance-profiling`](./skills/performance-profiling/SKILL.md) |
| **Mobile Apps (Android / iOS)** | [`mobile-developer`](./agents/mobile-developer.md) | [`mobile-design-android`](./skills/mobile-design-android/SKILL.md), [`mobile-design-ios`](./skills/mobile-design-ios/SKILL.md) |
| **Project Planning & Strategy** | [`project-planner`](./agents/project-planner.md) / [`product-owner`](./agents/product-owner.md) | [`plan-writing`](./skills/plan-writing/SKILL.md), [`brainstorming`](./skills/brainstorming/SKILL.md), [`architecture`](./skills/architecture/SKILL.md) |
| **Customer Backend & Ingestion** | [`customer-backend-specialist`](./agents/customer-backend-specialist.md) | [`api-patterns`](./skills/api-patterns/SKILL.md), [`smb-data-ingestion`](./skills/smb-data-ingestion/SKILL.md), [`digital-twin-voice`](./skills/digital-twin-voice/SKILL.md) |

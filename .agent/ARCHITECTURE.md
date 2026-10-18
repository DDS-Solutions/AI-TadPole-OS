> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Documentation / ARCHITECTURE
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[ARCHITECTURE]`)

# Antigravity Kit Architecture

> Comprehensive AI Agent Capability Expansion Toolkit for Sovereign Swarm Intelligence

## 🧠 Core Intelligence & Routing

- **Intelligent Routing**: Located at [`intelligent-routing`](file:///D:/TadpoleOS-Dev/.agent/skills/intelligent-routing/SKILL.md). This is the primary logic gate for autonomous agent selection, multi-domain classification, and task routing.
- **Skills Standard**: All modules conform to the open [Agent Skills Specification](https://agentskills.io) with progressive disclosure (L1 YAML metadata, L2 instructions, L3 references/assets). Comprehensive creation guide is available at [`doc.md`](file:///D:/TadpoleOS-Dev/.agent/skills/doc.md).

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
| [`backend-specialist`](file:///D:/TadpoleOS-Dev/.agent/agents/backend-specialist.md) | Backend architect specializing in high-availability, security-first infrastructure. Node.js, Python, API, DB, DevSecOps. | [`clean-code`](file:///D:/TadpoleOS-Dev/.agent/skills/clean-code/SKILL.md), [`nodejs-best-practices`](file:///D:/TadpoleOS-Dev/.agent/skills/nodejs-best-practices/SKILL.md), [`python-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/python-patterns/SKILL.md), [`api-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/api-patterns/SKILL.md), [`database-design`](file:///D:/TadpoleOS-Dev/.agent/skills/database-design/SKILL.md), [`rust-pro`](file:///D:/TadpoleOS-Dev/.agent/skills/rust-pro/SKILL.md), [`architecture`](file:///D:/TadpoleOS-Dev/.agent/skills/architecture/SKILL.md), [`server-management`](file:///D:/TadpoleOS-Dev/.agent/skills/server-management/SKILL.md) |
| [`code-archaeologist`](file:///D:/TadpoleOS-Dev/.agent/agents/code-archaeologist.md) | Legacy code expert. Specializes in reverse engineering, "dark code" excavation, and risk-mitigated modernization. | [`code-review-graph`](file:///D:/TadpoleOS-Dev/.agent/skills/code-review-graph/SKILL.md), [`simplify-code`](file:///D:/TadpoleOS-Dev/.agent/skills/simplify-code/SKILL.md), [`verify-changes`](file:///D:/TadpoleOS-Dev/.agent/skills/verify-changes/SKILL.md), [`architecture`](file:///D:/TadpoleOS-Dev/.agent/skills/architecture/SKILL.md), [`clean-code`](file:///D:/TadpoleOS-Dev/.agent/skills/clean-code/SKILL.md) |
| [`compliance-officer`](file:///D:/TadpoleOS-Dev/.agent/agents/compliance-officer.md) | Legal & Compliance Architect. Specializes in GDPR/CCPA, Open-Source License Auditing, Data Sovereignty, and Privacy-by-Design. | [`vulnerability-scanner`](file:///D:/TadpoleOS-Dev/.agent/skills/vulnerability-scanner/SKILL.md), [`pii-redaction`](file:///D:/TadpoleOS-Dev/.agent/skills/pii-redaction/SKILL.md), [`architecture`](file:///D:/TadpoleOS-Dev/.agent/skills/architecture/SKILL.md), [`web-design-guidelines`](file:///D:/TadpoleOS-Dev/.agent/skills/web-design-guidelines/SKILL.md) |
| [`customer-backend-specialist`](file:///D:/TadpoleOS-Dev/.agent/agents/customer-backend-specialist.md) | Customer-facing backend specialist focused on production safety, API compatibility, privacy, migrations, observability, idempotency, supportability, and rollout risk. | [`clean-code`](file:///D:/TadpoleOS-Dev/.agent/skills/clean-code/SKILL.md), [`api-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/api-patterns/SKILL.md), [`database-design`](file:///D:/TadpoleOS-Dev/.agent/skills/database-design/SKILL.md), [`smb-data-ingestion`](file:///D:/TadpoleOS-Dev/.agent/skills/smb-data-ingestion/SKILL.md), [`digital-twin-voice`](file:///D:/TadpoleOS-Dev/.agent/skills/digital-twin-voice/SKILL.md), [`architecture`](file:///D:/TadpoleOS-Dev/.agent/skills/architecture/SKILL.md), [`server-management`](file:///D:/TadpoleOS-Dev/.agent/skills/server-management/SKILL.md) |
| [`database-architect`](file:///D:/TadpoleOS-Dev/.agent/agents/database-architect.md) | Database architect specializing in schema design, query optimization, and zero-downtime migrations. | [`database-design`](file:///D:/TadpoleOS-Dev/.agent/skills/database-design/SKILL.md), [`performance-profiling`](file:///D:/TadpoleOS-Dev/.agent/skills/performance-profiling/SKILL.md), [`architecture`](file:///D:/TadpoleOS-Dev/.agent/skills/architecture/SKILL.md) |
| [`debugger`](file:///D:/TadpoleOS-Dev/.agent/agents/debugger.md) | Root Cause Analysis expert. Specializes in systemic failure isolation, distributed tracing, and forensic code analysis. | [`systematic-debugging`](file:///D:/TadpoleOS-Dev/.agent/skills/systematic-debugging/SKILL.md), [`performance-profiling`](file:///D:/TadpoleOS-Dev/.agent/skills/performance-profiling/SKILL.md), [`powershell-windows`](file:///D:/TadpoleOS-Dev/.agent/skills/powershell-windows/SKILL.md), [`bash-linux`](file:///D:/TadpoleOS-Dev/.agent/skills/bash-linux/SKILL.md) |
| [`devops-engineer`](file:///D:/TadpoleOS-Dev/.agent/agents/devops-engineer.md) | Infrastructure and Operations architect. Specialist in CI/CD, IaC, Global Distribution, and Disaster Recovery. | [`deployment-procedures`](file:///D:/TadpoleOS-Dev/.agent/skills/deployment-procedures/SKILL.md), [`server-management`](file:///D:/TadpoleOS-Dev/.agent/skills/server-management/SKILL.md), [`vulnerability-scanner`](file:///D:/TadpoleOS-Dev/.agent/skills/vulnerability-scanner/SKILL.md), [`bash-linux`](file:///D:/TadpoleOS-Dev/.agent/skills/bash-linux/SKILL.md), [`powershell-windows`](file:///D:/TadpoleOS-Dev/.agent/skills/powershell-windows/SKILL.md) |
| [`documentation-writer`](file:///D:/TadpoleOS-Dev/.agent/agents/documentation-writer.md) | Technical Knowledge Architect. Specializes in API specifications, Architecture Decision Records (ADRs), and the systemic prevention of information decay. | [`documentation-templates`](file:///D:/TadpoleOS-Dev/.agent/skills/documentation-templates/SKILL.md), [`architecture`](file:///D:/TadpoleOS-Dev/.agent/skills/architecture/SKILL.md), [`api-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/api-patterns/SKILL.md), [`i18n-localization`](file:///D:/TadpoleOS-Dev/.agent/skills/i18n-localization/SKILL.md) |
| [`explorer-agent`](file:///D:/TadpoleOS-Dev/.agent/agents/explorer-agent.md) | Codebase Discovery & Architectural Intelligence. Specializes in mapping critical paths, identifying technical debt, and detecting documentation-code drift. | [`clean-code`](file:///D:/TadpoleOS-Dev/.agent/skills/clean-code/SKILL.md), [`systematic-debugging`](file:///D:/TadpoleOS-Dev/.agent/skills/systematic-debugging/SKILL.md), [`explorer-scout`](file:///D:/TadpoleOS-Dev/.agent/skills/explorer-scout/SKILL.md), [`code-review-graph`](file:///D:/TadpoleOS-Dev/.agent/skills/code-review-graph/SKILL.md), [`architecture`](file:///D:/TadpoleOS-Dev/.agent/skills/architecture/SKILL.md) |
| [`frontend-specialist`](file:///D:/TadpoleOS-Dev/.agent/agents/frontend-specialist.md) | Senior Frontend Architect. Expert in React/Next.js, UI/UX Psychology, Accessibility, and Tailwind v4. Specializes in high-performance, bespoke interfaces. | [`clean-code`](file:///D:/TadpoleOS-Dev/.agent/skills/clean-code/SKILL.md), [`nextjs-react-expert`](file:///D:/TadpoleOS-Dev/.agent/skills/nextjs-react-expert/SKILL.md), [`frontend-design`](file:///D:/TadpoleOS-Dev/.agent/skills/frontend-design/SKILL.md), [`tailwind-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/tailwind-patterns/SKILL.md), [`web-design-guidelines`](file:///D:/TadpoleOS-Dev/.agent/skills/web-design-guidelines/SKILL.md), [`performance-profiling`](file:///D:/TadpoleOS-Dev/.agent/skills/performance-profiling/SKILL.md) |
| [`game-developer`](file:///D:/TadpoleOS-Dev/.agent/agents/game-developer.md) | Game development expert. Unity, Godot, Phaser. Mechanics, Optimization, Multiplayer. | [`clean-code`](file:///D:/TadpoleOS-Dev/.agent/skills/clean-code/SKILL.md), [`game-development`](file:///D:/TadpoleOS-Dev/.agent/skills/game-development/SKILL.md) |
| [`growth-engineer`](file:///D:/TadpoleOS-Dev/.agent/agents/growth-engineer.md) | Data Scientist & Growth Architect. Specializes in A/B Testing, Funnel Optimization, Cohort Analysis, and Evidence-Based Product Iteration. | [`seo-fundamentals`](file:///D:/TadpoleOS-Dev/.agent/skills/seo-fundamentals/SKILL.md), [`frontend-design`](file:///D:/TadpoleOS-Dev/.agent/skills/frontend-design/SKILL.md), [`nextjs-react-expert`](file:///D:/TadpoleOS-Dev/.agent/skills/nextjs-react-expert/SKILL.md), [`web-design-guidelines`](file:///D:/TadpoleOS-Dev/.agent/skills/web-design-guidelines/SKILL.md) |
| [`mobile-developer`](file:///D:/TadpoleOS-Dev/.agent/agents/mobile-developer.md) | Mobile expert. Jetpack Compose, SwiftUI, React Native, Flutter. iOS/Android nuances. | [`clean-code`](file:///D:/TadpoleOS-Dev/.agent/skills/clean-code/SKILL.md), [`mobile-design-android`](file:///D:/TadpoleOS-Dev/.agent/skills/mobile-design-android/SKILL.md), [`mobile-design-ios`](file:///D:/TadpoleOS-Dev/.agent/skills/mobile-design-ios/SKILL.md) |
| [`nexus-engineer`](file:///D:/TadpoleOS-Dev/.agent/agents/nexus-engineer.md) | Fused Master System Architect, Principal QA, and Chief Security Auditor. Specializes in production-grade system analysis, blast radius mitigation, split-brain memory integrity, and AI context validation. | [`clean-code`](file:///D:/TadpoleOS-Dev/.agent/skills/clean-code/SKILL.md), [`code-review`](file:///D:/TadpoleOS-Dev/.agent/skills/code-review/SKILL.md), [`code-review-checklist`](file:///D:/TadpoleOS-Dev/.agent/skills/code-review-checklist/SKILL.md), [`code-review-graph`](file:///D:/TadpoleOS-Dev/.agent/skills/code-review-graph/SKILL.md), [`architecture`](file:///D:/TadpoleOS-Dev/.agent/skills/architecture/SKILL.md), [`improve-codebase-architecture`](file:///D:/TadpoleOS-Dev/.agent/skills/improve-codebase-architecture/SKILL.md), [`vulnerability-scanner`](file:///D:/TadpoleOS-Dev/.agent/skills/vulnerability-scanner/SKILL.md), [`testing-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/testing-patterns/SKILL.md), [`verify-changes`](file:///D:/TadpoleOS-Dev/.agent/skills/verify-changes/SKILL.md), [`powershell-windows`](file:///D:/TadpoleOS-Dev/.agent/skills/powershell-windows/SKILL.md), [`rust-pro`](file:///D:/TadpoleOS-Dev/.agent/skills/rust-pro/SKILL.md), [`nextjs-react-expert`](file:///D:/TadpoleOS-Dev/.agent/skills/nextjs-react-expert/SKILL.md) |
| [`orchestrator`](file:///D:/TadpoleOS-Dev/.agent/agents/orchestrator.md) | Master Agent for Multi-Agent Coordination. Decomposes complex objectives into executable pipelines, invokes specialists, and synthesizes disparate outputs into a unified Truth. | [`parallel-agents`](file:///D:/TadpoleOS-Dev/.agent/skills/parallel-agents/SKILL.md), [`coordinator-mode`](file:///D:/TadpoleOS-Dev/.agent/skills/coordinator-mode/SKILL.md), [`architecture`](file:///D:/TadpoleOS-Dev/.agent/skills/architecture/SKILL.md), [`wayfinder`](file:///D:/TadpoleOS-Dev/.agent/skills/wayfinder/SKILL.md), [`handoff`](file:///D:/TadpoleOS-Dev/.agent/skills/handoff/SKILL.md), [`lint-and-validate`](file:///D:/TadpoleOS-Dev/.agent/skills/lint-and-validate/SKILL.md) |
| [`penetration-tester`](file:///D:/TadpoleOS-Dev/.agent/agents/penetration-tester.md) | Offensive Security Engineer. Specializes in vulnerability research, red-team tactics, and the exploitation of logical flaws in APIs and Infrastructure-as-Code. | [`red-team-tactics`](file:///D:/TadpoleOS-Dev/.agent/skills/red-team-tactics/SKILL.md), [`vulnerability-scanner`](file:///D:/TadpoleOS-Dev/.agent/skills/vulnerability-scanner/SKILL.md), [`api-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/api-patterns/SKILL.md) |
| [`performance-optimizer`](file:///D:/TadpoleOS-Dev/.agent/agents/performance-optimizer.md) | Full-Stack Performance Architect. Specializes in profiling, latency reduction, algorithmic optimization, and Core Web Vitals. Operates on the principle of "Evidence over Intuition." | [`performance-profiling`](file:///D:/TadpoleOS-Dev/.agent/skills/performance-profiling/SKILL.md), [`seo-fundamentals`](file:///D:/TadpoleOS-Dev/.agent/skills/seo-fundamentals/SKILL.md), [`architecture`](file:///D:/TadpoleOS-Dev/.agent/skills/architecture/SKILL.md) |
| [`product-manager`](file:///D:/TadpoleOS-Dev/.agent/agents/product-manager.md) | Product Requirements Engineer. Specializes in User Experience (UX) strategy, Requirement Decomposition, and Value-Driven Prioritization. | [`plan-writing`](file:///D:/TadpoleOS-Dev/.agent/skills/plan-writing/SKILL.md), [`brainstorming`](file:///D:/TadpoleOS-Dev/.agent/skills/brainstorming/SKILL.md), [`web-design-guidelines`](file:///D:/TadpoleOS-Dev/.agent/skills/web-design-guidelines/SKILL.md) |
| [`product-owner`](file:///D:/TadpoleOS-Dev/.agent/agents/product-owner.md) | Strategic Product Governor. Specializes in Value Stream Mapping, ROI Analysis, Roadmap Orchestration, and Strategic Trade-offs. | [`plan-writing`](file:///D:/TadpoleOS-Dev/.agent/skills/plan-writing/SKILL.md), [`brainstorming`](file:///D:/TadpoleOS-Dev/.agent/skills/brainstorming/SKILL.md), [`architecture`](file:///D:/TadpoleOS-Dev/.agent/skills/architecture/SKILL.md) |
| [`project-planner`](file:///D:/TadpoleOS-Dev/.agent/agents/project-planner.md) | Technical Architect & Strategist. Specializes in task decomposition, system design, and the creation of executable blueprints. Converts "What" (PRD) into "How" (Atomic Tasks). | [`plan-writing`](file:///D:/TadpoleOS-Dev/.agent/skills/plan-writing/SKILL.md), [`architecture`](file:///D:/TadpoleOS-Dev/.agent/skills/architecture/SKILL.md) |
| [`qa-automation-engineer`](file:///D:/TadpoleOS-Dev/.agent/agents/qa-automation-engineer.md) | Quality Assurance & Automation Architect. Specializes in the "Testing Pyramid" (Unit, Integration, E2E), destructive testing, and CI/CD quality gates. The final arbiter of "Done." | [`webapp-testing`](file:///D:/TadpoleOS-Dev/.agent/skills/webapp-testing/SKILL.md), [`testing-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/testing-patterns/SKILL.md), [`red-team-tactics`](file:///D:/TadpoleOS-Dev/.agent/skills/red-team-tactics/SKILL.md) |
| [`release-manager`](file:///D:/TadpoleOS-Dev/.agent/agents/release-manager.md) | Delivery & Versioning Architect. Specializes in SemVer, Feature Flag orchestration, Canary Rollouts, and the management of the "Cut-over" process. | [`deployment-procedures`](file:///D:/TadpoleOS-Dev/.agent/skills/deployment-procedures/SKILL.md), [`verify-changes`](file:///D:/TadpoleOS-Dev/.agent/skills/verify-changes/SKILL.md) |
| [`security-auditor`](file:///D:/TadpoleOS-Dev/.agent/agents/security-auditor.md) | Defensive Security Architect & Auditor. Specializes in Zero Trust architecture, compliance (OWASP, NIST), systemic hardening, and vulnerability management. | [`vulnerability-scanner`](file:///D:/TadpoleOS-Dev/.agent/skills/vulnerability-scanner/SKILL.md), [`red-team-tactics`](file:///D:/TadpoleOS-Dev/.agent/skills/red-team-tactics/SKILL.md), [`architecture`](file:///D:/TadpoleOS-Dev/.agent/skills/architecture/SKILL.md) |
| [`seo-specialist`](file:///D:/TadpoleOS-Dev/.agent/agents/seo-specialist.md) | Visibility Architect. Specializes in Search Engine Optimization (SEO) and Generative/Answer Engine Optimization (GEO/AEO). Expert in Semantic HTML, Schema.org, and Entity-based content strategy. | [`seo-fundamentals`](file:///D:/TadpoleOS-Dev/.agent/skills/seo-fundamentals/SKILL.md), [`geo-fundamentals`](file:///D:/TadpoleOS-Dev/.agent/skills/geo-fundamentals/SKILL.md) |
| [`sre-architect`](file:///D:/TadpoleOS-Dev/.agent/agents/sre-architect.md) | Site Reliability Engineer. Specializes in SLO/SLI management, incident response, observability, and high-availability system hardening. | [`server-management`](file:///D:/TadpoleOS-Dev/.agent/skills/server-management/SKILL.md), [`performance-profiling`](file:///D:/TadpoleOS-Dev/.agent/skills/performance-profiling/SKILL.md), [`red-team-tactics`](file:///D:/TadpoleOS-Dev/.agent/skills/red-team-tactics/SKILL.md) |
| [`tadpole-backend-specialist`](file:///D:/TadpoleOS-Dev/.agent/agents/tadpole-backend-specialist.md) | Tadpole OS backend specialist for Rust, Axum, Tokio, sqlx, SQLite, AppState, agent registry, runner lifecycle, WebSocket events, and backend contract integrity. | [`clean-code`](file:///D:/TadpoleOS-Dev/.agent/skills/clean-code/SKILL.md), [`rust-pro`](file:///D:/TadpoleOS-Dev/.agent/skills/rust-pro/SKILL.md), [`api-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/api-patterns/SKILL.md), [`database-design`](file:///D:/TadpoleOS-Dev/.agent/skills/database-design/SKILL.md), [`architecture`](file:///D:/TadpoleOS-Dev/.agent/skills/architecture/SKILL.md), [`server-management`](file:///D:/TadpoleOS-Dev/.agent/skills/server-management/SKILL.md) |
| [`tadpole-frontend-specialist`](file:///D:/TadpoleOS-Dev/.agent/agents/tadpole-frontend-specialist.md) | Tadpole OS frontend specialist for React, Vite, TypeScript, Zustand, Tailwind v4, WebSockets integration, Tauri bridge, and frontend-backend contracts. | [`clean-code`](file:///D:/TadpoleOS-Dev/.agent/skills/clean-code/SKILL.md), [`nextjs-react-expert`](file:///D:/TadpoleOS-Dev/.agent/skills/nextjs-react-expert/SKILL.md), [`frontend-design`](file:///D:/TadpoleOS-Dev/.agent/skills/frontend-design/SKILL.md), [`tailwind-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/tailwind-patterns/SKILL.md), [`web-design-guidelines`](file:///D:/TadpoleOS-Dev/.agent/skills/web-design-guidelines/SKILL.md), [`performance-profiling`](file:///D:/TadpoleOS-Dev/.agent/skills/performance-profiling/SKILL.md) |
| [`test-engineer`](file:///D:/TadpoleOS-Dev/.agent/agents/test-engineer.md) | TDD & Logic Verification Expert. Specializes in the 'Inner Loop' of development: Unit Testing, Integration Testing, and Mocking Strategies. | [`tdd-workflow`](file:///D:/TadpoleOS-Dev/.agent/skills/tdd-workflow/SKILL.md), [`testing-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/testing-patterns/SKILL.md) |
| [`ux-designer`](file:///D:/TadpoleOS-Dev/.agent/agents/ux-designer.md) | Product & UX Designer. Specializes in User Psychology, Interaction Design, User Journey Mapping, and Design Systems. | [`web-design-guidelines`](file:///D:/TadpoleOS-Dev/.agent/skills/web-design-guidelines/SKILL.md), [`frontend-design`](file:///D:/TadpoleOS-Dev/.agent/skills/frontend-design/SKILL.md), [`tailwind-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/tailwind-patterns/SKILL.md) |

---

## 🧩 Skills (67)

Modular knowledge domains adhering to the [Agent Skills Specification](https://agentskills.io). Loaded on-demand via progressive disclosure based on prompt intent and task routing.
### Frontend & UI (6)

| Skill | Description |
| ----- | ----------- |
| [`nextjs-react-expert`](file:///D:/TadpoleOS-Dev/.agent/skills/nextjs-react-expert/SKILL.md) | React, Next.js, and Vite SPA performance optimization. Covers waterfall elimination, bundle size reduction, React 19 hooks, and client-side desktop performance. |
| [`frontend-design`](file:///D:/TadpoleOS-Dev/.agent/skills/frontend-design/SKILL.md) | Design thinking and decision-making for web UI. Use when designing components, layouts, color schemes, typography, or creating aesthetic interfaces. Teaches principles, not fixed values. |
| [`tailwind-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/tailwind-patterns/SKILL.md) | Tailwind CSS v4 principles. CSS-first configuration, container queries, modern patterns, design token architecture. |
| [`web-design-guidelines`](file:///D:/TadpoleOS-Dev/.agent/skills/web-design-guidelines/SKILL.md) | Review UI code for Web Interface Guidelines compliance. Focus outlines, touch targets, accessibility (WCAG AA), typography, and interactive affordances. |
| [`zustand-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/zustand-patterns/SKILL.md) | Zustand state store architecture, slice patterns, selector memoization with useShallow, and outside-component actions. |
| [`prototype`](file:///D:/TadpoleOS-Dev/.agent/skills/prototype/SKILL.md) | Builds minimal, low-fidelity executable spikes or stub components to resolve 'how should it look/behave' questions before production coding. |

### Backend & Systems (7)

| Skill | Description |
| ----- | ----------- |
| [`rust-pro`](file:///D:/TadpoleOS-Dev/.agent/skills/rust-pro/SKILL.md) | Master Rust 1.75+ with modern async patterns, advanced type system features, and production-ready systems programming. Expert in Tokio, Axum, and zero-cost abstractions. |
| [`tauri-pro`](file:///D:/TadpoleOS-Dev/.agent/skills/tauri-pro/SKILL.md) | Master Tauri v1/v2 desktop development with Rust commands, IPC bridge, security capabilities, window state, and event bus. |
| [`api-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/api-patterns/SKILL.md) | API design principles and decision-making. REST vs GraphQL vs tRPC selection, response formats, versioning, pagination. |
| [`nodejs-best-practices`](file:///D:/TadpoleOS-Dev/.agent/skills/nodejs-best-practices/SKILL.md) | Node.js development principles and decision-making. Framework selection, async patterns, security, and architecture. Teaches thinking, not copying. |
| [`python-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/python-patterns/SKILL.md) | Python development principles and decision-making. Framework selection, async patterns, type hints, and project structure. |
| [`server-management`](file:///D:/TadpoleOS-Dev/.agent/skills/server-management/SKILL.md) | Server management principles and decision-making. Process supervision, systemd units, PM2 clustering, health monitoring, and scaling strategies. |
| [`deployment-procedures`](file:///D:/TadpoleOS-Dev/.agent/skills/deployment-procedures/SKILL.md) | Production deployment principles and decision-making. Safe deployment workflows, rollback strategies, and verification. Teaches thinking, not scripts. |

### Database & Vector Memory (4)

| Skill | Description |
| ----- | ----------- |
| [`database-design`](file:///D:/TadpoleOS-Dev/.agent/skills/database-design/SKILL.md) | Database design principles, schema modeling, migration strategies, indexing, and ORM selection. Covers SQLite/sqlx, PostgreSQL/Neon, Turso, LanceDB vector memory, Prisma, and Drizzle. |
| [`vector-memory-lancedb`](file:///D:/TadpoleOS-Dev/.agent/skills/vector-memory-lancedb/SKILL.md) | Dense vector memory, LanceDB table schemas, embedding pipelines, cosine similarity deduplication, and semantic RAG retrieval. |
| [`domain-modeling`](file:///D:/TadpoleOS-Dev/.agent/skills/domain-modeling/SKILL.md) | Actively builds and sharpens the project's domain model, ubiquitous glossary (CONTEXT.md / docs/GLOSSARY.md), and architectural decision records (docs/adr/). |
| [`smb-data-ingestion`](file:///D:/TadpoleOS-Dev/.agent/skills/smb-data-ingestion/SKILL.md) | Parses and validates messy SMB data exports (CSV, Excel, Quickbooks PDFs, legacy SQL) into clean JSON-Schema & SQLite state for AI Digital Twins. |

### Mobile Development (2)

| Skill | Description |
| ----- | ----------- |
| [`mobile-design-android`](file:///D:/TadpoleOS-Dev/.agent/skills/mobile-design-android/SKILL.md) | Mobile-first Android design thinking, Jetpack Compose, Material 3 (M3), Compose Navigation 3, Compose Material 3 Adaptive layouts, StateFlow / ViewModel state management, and Android platform conventions. Synchronized with official Google Developer documentation. |
| [`mobile-design-ios`](file:///D:/TadpoleOS-Dev/.agent/skills/mobile-design-ios/SKILL.md) | Mobile-first iOS design thinking, SwiftUI, UIKit, Apple Human Interface Guidelines (HIG), SF Symbols, NavigationStack/SplitView, @Observable macro, SwiftData/Core Data, and iOS platform conventions. |

### Security, Governance & Red Teaming (5)

| Skill | Description |
| ----- | ----------- |
| [`vulnerability-scanner`](file:///D:/TadpoleOS-Dev/.agent/skills/vulnerability-scanner/SKILL.md) | Advanced vulnerability analysis principles. OWASP Agentic Top 10 (2026), MITRE ATLAS, Supply Chain Security, secret scanning, and attack surface mapping. |
| [`red-team-tactics`](file:///D:/TadpoleOS-Dev/.agent/skills/red-team-tactics/SKILL.md) | Red team tactics principles based on MITRE ATT&CK v19.2 and MITRE ATLAS v2026.07. Stealth, defense impairment, and agentic swarm attack phases. |
| [`pii-redaction`](file:///D:/TadpoleOS-Dev/.agent/skills/pii-redaction/SKILL.md) | Zero-Trust PII sanitizer that redacts customer credit cards, phone numbers, SSNs, emails, and API keys before data is sent to cloud LLM providers. |
| [`a2a-protocol`](file:///D:/TadpoleOS-Dev/.agent/skills/a2a-protocol/SKILL.md) | Agent-to-Agent (A2A) economic protocol, Web3 vault settlement, capability key rotation, and cryptographic oversight signatures. |
| [`adg-governance`](file:///D:/TadpoleOS-Dev/.agent/skills/adg-governance/SKILL.md) | Active Documentation Guard (ADG v2) protocol, machine-maintained witness manifests, AST symbol blast radius analysis, and parity guard compliance. |

### Testing, QA & Verification (8)

| Skill | Description |
| ----- | ----------- |
| [`testing-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/testing-patterns/SKILL.md) | Testing patterns and principles. Unit, integration, mocking strategies, Arrange-Act-Assert (AAA), and multi-ecosystem test runners. |
| [`webapp-testing`](file:///D:/TadpoleOS-Dev/.agent/skills/webapp-testing/SKILL.md) | Web application testing principles. E2E, Playwright, deep audit strategies. |
| [`tdd-workflow`](file:///D:/TadpoleOS-Dev/.agent/skills/tdd-workflow/SKILL.md) | Test-Driven Development workflow principles. RED-GREEN-REFACTOR cycle, writing failing tests first, and enforcing behavioral verification. |
| [`verify-changes`](file:///D:/TadpoleOS-Dev/.agent/skills/verify-changes/SKILL.md) | Prove code works by running it, not just checking it exists. Verification through execution rather than inspection. Use after writing or modifying code to ensure it actually functions correctly. |
| [`code-review`](file:///D:/TadpoleOS-Dev/.agent/skills/code-review/SKILL.md) | Two-axis parallel code review comparing HEAD against a fixed point. Axis 1 (Standards) audits repo conventions and Fowler smells; Axis 2 (Spec) verifies line-by-line requirements coverage. |
| [`code-review-checklist`](file:///D:/TadpoleOS-Dev/.agent/skills/code-review-checklist/SKILL.md) | Systematic code review audit engine and master rubric across logic, security hardening, performance, clean-code adherence, and test coverage. |
| [`code-review-graph`](file:///D:/TadpoleOS-Dev/.agent/skills/code-review-graph/SKILL.md) | Token-efficient code review using Tree-sitter AST graphs and MCP. Reduces AI assistant token usage by up to 49x by computing blast radius instead of reading entire folders. |
| [`lint-and-validate`](file:///D:/TadpoleOS-Dev/.agent/skills/lint-and-validate/SKILL.md) | Automatic quality control, linting, and static analysis procedures. Use after every code modification to ensure syntax correctness and project standards. Triggers onKeywords: lint, format, check, validate, types, static analysis. |

### Debugging & Diagnostics (2)

| Skill | Description |
| ----- | ----------- |
| [`systematic-debugging`](file:///D:/TadpoleOS-Dev/.agent/skills/systematic-debugging/SKILL.md) | 4-phase systematic debugging methodology with root cause analysis and evidence-based verification. Use when debugging complex issues. |
| [`performance-profiling`](file:///D:/TadpoleOS-Dev/.agent/skills/performance-profiling/SKILL.md) | Performance profiling principles. Measurement, analysis, and optimization techniques. Core Web Vitals, Lighthouse audits, flame graphs, and bundle inspection. |

### Architecture, Planning & Reasoning (9)

| Skill | Description |
| ----- | ----------- |
| [`architecture`](file:///D:/TadpoleOS-Dev/.agent/skills/architecture/SKILL.md) | Architectural decision-making framework. Requirements analysis, trade-off evaluation, ADR documentation. Use when making architecture decisions or analyzing system design. |
| [`improve-codebase-architecture`](file:///D:/TadpoleOS-Dev/.agent/skills/improve-codebase-architecture/SKILL.md) | Scans git commit hot spots for shallow modules, generates visual HTML reports with Mermaid diagrams, and drills into refactoring candidates. |
| [`app-builder`](file:///D:/TadpoleOS-Dev/.agent/skills/app-builder/SKILL.md) | Main application building orchestrator. Creates full-stack applications from natural language requests. Determines project type, selects tech stack, coordinates agents. |
| [`plan-writing`](file:///D:/TadpoleOS-Dev/.agent/skills/plan-writing/SKILL.md) | Structured task planning with clear breakdowns, dependencies, and verification criteria. Use when implementing features, refactoring, or any multi-step work. |
| [`brainstorming`](file:///D:/TadpoleOS-Dev/.agent/skills/brainstorming/SKILL.md) | Socratic questioning protocol and communication framework. Mandatory operational gateway for complex requests, ambiguous requirements, and architectural trade-off alignment. |
| [`aletheia-reasoning`](file:///D:/TadpoleOS-Dev/.agent/skills/aletheia-reasoning/SKILL.md) | Structured iterative reasoning framework for deep problem solving, featuring generator, verifier, reviser, and final proof stages. |
| [`to-spec`](file:///D:/TadpoleOS-Dev/.agent/skills/to-spec/SKILL.md) | Converts informal conversation context or brainstorms into formal, published specification files under docs/specs/. |
| [`wayfinder`](file:///D:/TadpoleOS-Dev/.agent/skills/wayfinder/SKILL.md) | Plan large multi-session initiatives wrapped in fog as a map of linked decision tickets, resolving them sequentially without context degradation. |
| [`world-model-synthesis`](file:///D:/TadpoleOS-Dev/.agent/skills/world-model-synthesis/SKILL.md) | Synthesizes programmatic executable world models (.tmp/world_model.py) and executes local graph search (BFS/A*) for deterministic agent task planning. |

### Multi-Agent Orchestration & Swarm (6)

| Skill | Description |
| ----- | ----------- |
| [`intelligent-routing`](file:///D:/TadpoleOS-Dev/.agent/skills/intelligent-routing/SKILL.md) | Automatic agent selection and intelligent task routing. Analyzes user requests, classifies intent and complexity, and automatically selects the best specialist agent(s) and skills without requiring explicit user mentions. |
| [`coordinator-mode`](file:///D:/TadpoleOS-Dev/.agent/skills/coordinator-mode/SKILL.md) | Advanced multi-agent orchestration with parallel workers, synthesis protocols, and coordinator lifecycle. Use when complex tasks require multiple agents working in parallel with intelligent result synthesis. |
| [`parallel-agents`](file:///D:/TadpoleOS-Dev/.agent/skills/parallel-agents/SKILL.md) | Multi-agent orchestration patterns. Use when multiple independent tasks can run with different domain expertise or when comprehensive analysis requires multiple perspectives. |
| [`behavioral-modes`](file:///D:/TadpoleOS-Dev/.agent/skills/behavioral-modes/SKILL.md) | Adaptive AI operational modes (BRAINSTORM, ORCHESTRATE, ALETHEIA, IMPLEMENT, DEBUG, REVIEW, TEACH, SHIP) and PEC handover protocols to eliminate mode bleed, premature execution, and conversational drift. |
| [`handoff`](file:///D:/TadpoleOS-Dev/.agent/skills/handoff/SKILL.md) | Compacts current session state into a clean, deterministic handoff document for incoming agents or multi-session state transfers. Prevents context drift and cold-start latency. |
| [`mission-analyst`](file:///D:/TadpoleOS-Dev/.agent/skills/mission-analyst/SKILL.md) | Agent 99 persona for automated post-mission debriefing. Extracts architectural wisdom and updates LONG_TERM_MEMORY.md. Triggers on: debrief, post-mission, extract wisdom, synthesize learnings. |

### Memory & Context Management (4)

| Skill | Description |
| ----- | ----------- |
| [`memory-system`](file:///D:/TadpoleOS-Dev/.agent/skills/memory-system/SKILL.md) | Persistent cross-session memory management. Enables agents to remember user preferences, project conventions, and past decisions across different sessions using a structured MEMORY.md index and topic files. |
| [`context-compression`](file:///D:/TadpoleOS-Dev/.agent/skills/context-compression/SKILL.md) | Manage and compress conversation context in long sessions. Detect when context is growing large, summarize completed work phases, archive old findings while preserving key decisions. Prevents context degradation. |
| [`digital-twin-voice`](file:///D:/TadpoleOS-Dev/.agent/skills/digital-twin-voice/SKILL.md) | Extracts SMB founder/brand tone, formality, and communication phrasing to calibrate AI Digital Twin persona artifacts. |
| [`gemma4-local`](file:///D:/TadpoleOS-Dev/.agent/skills/gemma4-local/SKILL.md) | Provides core guidelines and prompt formatting patterns for interacting with local gemma4 models. Triggers when gemma4 is used as the local LLM backend. |

### Code Engineering & Automation (4)

| Skill | Description |
| ----- | ----------- |
| [`clean-code`](file:///D:/TadpoleOS-Dev/.agent/skills/clean-code/SKILL.md) | Pragmatic coding standards: concise, direct, zero over-engineering, guard clauses, cognitive load minimization, and language idioms (Rust, TS, Python). |
| [`simplify-code`](file:///D:/TadpoleOS-Dev/.agent/skills/simplify-code/SKILL.md) | Reduce complexity of over-engineered code. Identify unnecessary abstractions, remove dead code, flatten deep nesting, and simplify logic while preserving behavior. |
| [`skillify`](file:///D:/TadpoleOS-Dev/.agent/skills/skillify/SKILL.md) | Auto-create new skills from repetitive workflows. When you notice yourself doing the same multi-step process repeatedly, extract it into a reusable SKILL.md that any agent can use. |
| [`batch-operations`](file:///D:/TadpoleOS-Dev/.agent/skills/batch-operations/SKILL.md) | Apply operations across multiple files simultaneously. Pattern-based bulk modifications, search-and-replace across codebases, consistent changes to many files at once. |

### CLI & Platform Tools (3)

| Skill | Description |
| ----- | ----------- |
| [`bash-linux`](file:///D:/TadpoleOS-Dev/.agent/skills/bash-linux/SKILL.md) | Bash/Linux terminal patterns. Critical commands, piping, error handling, scripting. Use when working on macOS or Linux systems. |
| [`powershell-windows`](file:///D:/TadpoleOS-Dev/.agent/skills/powershell-windows/SKILL.md) | PowerShell Windows patterns. Critical pitfalls, operator syntax, error handling, JSON serialization, and cmdlets. |
| [`mcp-builder`](file:///D:/TadpoleOS-Dev/.agent/skills/mcp-builder/SKILL.md) | MCP (Model Context Protocol) server building principles. Tool design, resource patterns, JSON-RPC 2.0 schemas, and transport standards. |

### Documentation, SEO & Growth (6)

| Skill | Description |
| ----- | ----------- |
| [`documentation-templates`](file:///D:/TadpoleOS-Dev/.agent/skills/documentation-templates/SKILL.md) | Standardized documentation protocols for humans and AI. Manages READMEs, API specs, ADRs, and AI-native context maps. |
| [`seo-fundamentals`](file:///D:/TadpoleOS-Dev/.agent/skills/seo-fundamentals/SKILL.md) | SEO fundamentals, E-E-A-T, Core Web Vitals, metadata schemas, OpenGraph tags, and Google algorithm principles. |
| [`geo-fundamentals`](file:///D:/TadpoleOS-Dev/.agent/skills/geo-fundamentals/SKILL.md) | Generative Engine Optimization execution protocol. Audits and injects structured schemas, statistical citation magnets, and E-E-A-T entity markers. |
| [`i18n-localization`](file:///D:/TadpoleOS-Dev/.agent/skills/i18n-localization/SKILL.md) | Internationalization (i18n) and localization (L10n) patterns. Detecting hardcoded strings, managing translations, locale files, RTL support, and Intl formatting. |
| [`explorer-scout`](file:///D:/TadpoleOS-Dev/.agent/skills/explorer-scout/SKILL.md) | Scouting and architectural analysis for GitHub repositories. Extracts manifest dependencies, licensing, and structural patterns. |
| [`research`](file:///D:/TadpoleOS-Dev/.agent/skills/research/SKILL.md) | Executes isolated research and investigation subagents on throwaway git branches to prevent cluttering main working state. |

### Specialized & Game Development (1)

| Skill | Description |
| ----- | ----------- |
| [`game-development`](file:///D:/TadpoleOS-Dev/.agent/skills/game-development/SKILL.md) | Game development orchestrator. Routes to platform-specific and genre-specific game engine reference guides. |

---

## 🔄 Workflows (20)

Deterministic slash command procedures. Invoke with `/command`. Each workflow defines strict preconditions, multi-agent routing, tool calling gates, and invariant verification pipelines.

| Command | Description & Operational Focus |
| ------- | ------------------------------- |
| [`/adversary`](file:///D:/TadpoleOS-Dev/.agent/workflows/adversary.md) | Sovereign Stress Validation. An adversarial workflow designed to uncover fragility, logic gaps, and "Edge-Case Collapse" through simulated attacks and chaos engineering. |
| [`/anneal`](file:///D:/TadpoleOS-Dev/.agent/workflows/anneal.md) | Annealing is the process of hardening the Sovereign OS instruction-set by iteratively resolving "faults." In this context, a fault is defined as a discrepancy where the system's output deviates from the intended sovereign logic. |
| [`/audit`](file:///D:/TadpoleOS-Dev/.agent/workflows/audit.md) | Sovereign Security & Compliance Audit. Executes deep system scans and verifies documentation parity. |
| [`/brainstorm`](file:///D:/TadpoleOS-Dev/.agent/workflows/brainstorm.md) | Structured brainstorming for projects and features. Explores multiple divergent options and evaluates them against Sovereign benchmarks before implementation. |
| [`/codify`](file:///D:/TadpoleOS-Dev/.agent/workflows/codify.md) | Knowledge Heritage Synthesis. Transforms successful ephemeral patterns and "intelligence wins" into permanent Sovereign Laws and Core Directives. |
| [`/create`](file:///D:/TadpoleOS-Dev/.agent/workflows/create.md) | Orchestrates the end-to-end creation of new applications. Integrates discovery, brainstorming, blueprinting, and multi-agent execution. |
| [`/debug`](file:///D:/TadpoleOS-Dev/.agent/workflows/debug.md) | Initiates the Sovereign Root Cause Analysis (RCA) protocol. Systematically executes a multi-phase forensic investigation, gathering telemetry, isolating fault vectors, and determining the Point of Divergence between code and directive. |
| [`/deploy`](file:///D:/TadpoleOS-Dev/.agent/workflows/deploy.md) | Production State Transition. Manages the movement of audited, hardened, and verified code into live environments via a strict integrity pipeline. |
| [`/enhance`](file:///D:/TadpoleOS-Dev/.agent/workflows/enhance.md) | Controlled evolutionary growth of existing applications. Ensures new features are integrated without introducing architectural drift or regressions. |
| [`/github_scout`](file:///D:/TadpoleOS-Dev/.agent/workflows/github_scout.md) | Project Intelligence Research. Performs deep architectural extraction of external repositories to identify innovation vectors and implementation patterns for the Sovereign Registry. |
| [`/migrate`](file:///D:/TadpoleOS-Dev/.agent/workflows/migrate.md) | Sovereign State Transition. Manages the deterministic evolution of the Registry (Database) schema to ensure zero data loss and zero structural drift. |
| [`/onboard`](file:///D:/TadpoleOS-Dev/.agent/workflows/onboard.md) | Agent Context Synchronization. Rapidly aligns new or returning AI agents with the current Sovereign state, eliminating "Cold-Start" drift and architectural hallucinations. |
| [`/orchestrate`](file:///D:/TadpoleOS-Dev/.agent/workflows/orchestrate.md) | Sovereign Governor for multi-agent orchestration. Coordinates specialized agents and systemic workflows to execute complex architectural transitions with zero drift. |
| [`/plan`](file:///D:/TadpoleOS-Dev/.agent/workflows/plan.md) | Sovereign Blueprinting. Coordinates the project-planner agent to translate brainstormed ideas into a deterministic architectural contract. No code is written during this phase |
| [`/preview`](file:///D:/TadpoleOS-Dev/.agent/workflows/preview.md) | Sovereign Sandbox Management. Handles the initialization, monitoring, and termination of the local simulation environment to verify architectural integrity before production deployment. |
| [`/refactor`](file:///D:/TadpoleOS-Dev/.agent/workflows/refactor.md) | Structural Hardening & Architectural Refinement. Focuses on reducing system entropy and decoupling the 4-Pillars without altering external behavior. |
| [`/report`](file:///D:/TadpoleOS-Dev/.agent/workflows/report.md) | Sovereign Intelligence Synthesis. Transforms raw swarm telemetry and architectural drift data into high-fidelity dossiers to drive the system's evolutionary loop. |
| [`/status`](file:///D:/TadpoleOS-Dev/.agent/workflows/status.md) | Sovereign Integrity Dashboard. Provides a real-time snapshot of the system's operational health and architectural alignment across the 4-Pillars |
| [`/test`](file:///D:/TadpoleOS-Dev/.agent/workflows/test.md) | Sovereign Verification Gate. A deterministic pipeline to prove architectural integrity and behavioral parity before any state transition to production. |
| [`/ui-ux-pro-max`](file:///D:/TadpoleOS-Dev/.agent/workflows/ui-ux-pro-max.md) | Sovereign Gateway Visual Protocol. An AI-driven architectural framework for generating deterministic design systems and high-fidelity UI/UX implementations. |

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
| [`mobile-design-android`](file:///D:/TadpoleOS-Dev/.agent/skills/mobile-design-android/SKILL.md) | 13 | Jetpack Compose M3, Adaptive layouts, touch psychology, mobile testing |
| [`api-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/api-patterns/SKILL.md) | 11 | REST, GraphQL, tRPC standards, pagination, API versioning & error schemas |
| [`game-development`](file:///D:/TadpoleOS-Dev/.agent/skills/game-development/SKILL.md) | 11 | Platform & genre engine guides, loop patterns, physics & math references |
| [`nextjs-react-expert`](file:///D:/TadpoleOS-Dev/.agent/skills/nextjs-react-expert/SKILL.md) | 11 | React 19 rules, Vite SPA optimization, rule converter, performance auditing |
| [`frontend-design`](file:///D:/TadpoleOS-Dev/.agent/skills/frontend-design/SKILL.md) | 9 | Accessibility audit, UX review, design tokens, micro-interactions |
| [`app-builder`](file:///D:/TadpoleOS-Dev/.agent/skills/app-builder/SKILL.md) | 7 | Full-stack scaffolding, tech stack matrix, starter kit automation |
| [`database-design`](file:///D:/TadpoleOS-Dev/.agent/skills/database-design/SKILL.md) | 7 | SQLite/sqlx schemas, indexing rubrics, migrations, query optimization |
| [`architecture`](file:///D:/TadpoleOS-Dev/.agent/skills/architecture/SKILL.md) | 6 | ADR templates, blast radius mitigation, trade-off evaluation matrices |
| [`mobile-design-ios`](file:///D:/TadpoleOS-Dev/.agent/skills/mobile-design-ios/SKILL.md) | 5 | SwiftUI, Apple HIG guidelines, navigation stacks, iOS validator |

---

## 🛠️ Master Validation & Execution Scripts

Deterministic Layer 3 Python scripts in `execution/` enforce sovereign system integrity across code, documentation, and agent specifications.

### Master Scripts Matrix

| Script | Purpose | When to Use |
| ------ | ------- | ----------- |
| [`execution/checklist.py`](file:///D:/TadpoleOS-Dev/execution/checklist.py) | Priority-based validation (P0: Security & Lint; P1: Docs & Schema; P2: Tests & UX) | Development cycles, pre-commit gate |
| [`execution/verify_all.py`](file:///D:/TadpoleOS-Dev/execution/verify_all.py) | Full verification release suite (Checklist + Lighthouse CWV, Playwright E2E, Bundle, i18n) | Pre-deployment, release builds |
| [`execution/parity_guard.py`](file:///D:/TadpoleOS-Dev/execution/parity_guard.py) | Architectural documentation parity, tag synchronization, and env var verification | All commits, CI pipeline gate |
| [`execution/verify_skill_specs.py`](file:///D:/TadpoleOS-Dev/execution/verify_skill_specs.py) | Open-spec validation for all 67 skills, 29 subagents, and 20 workflows | Skill authoring, agent updates |
| [`execution/verify_doc_links.py`](file:///D:/TadpoleOS-Dev/execution/verify_doc_links.py) | Complete cross-reference and markdown link integrity guard | Documentation updates |

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
| **Master Execution Scripts** | 5 | [`execution/`](file:///D:/TadpoleOS-Dev/execution) |
| **Specification Conformance** | 100% (116/116 passed) | `python execution/verify_skill_specs.py` |
| **Documentation Link Integrity** | 100% healthy | `python execution/verify_doc_links.py` |

---

## 🔗 Quick Reference & Routing Map

| Operational Need | Primary Specialist Agent | Bound Skills |
| ---------------- | ------------------------ | ------------ |
| **Full-Stack Desktop App** | [`tadpole-frontend-specialist`](file:///D:/TadpoleOS-Dev/.agent/agents/tadpole-frontend-specialist.md) / [`tadpole-backend-specialist`](file:///D:/TadpoleOS-Dev/.agent/agents/tadpole-backend-specialist.md) | [`tauri-pro`](file:///D:/TadpoleOS-Dev/.agent/skills/tauri-pro/SKILL.md), [`nextjs-react-expert`](file:///D:/TadpoleOS-Dev/.agent/skills/nextjs-react-expert/SKILL.md), [`rust-pro`](file:///D:/TadpoleOS-Dev/.agent/skills/rust-pro/SKILL.md) |
| **Rust Axum & Tauri IPC** | [`tadpole-backend-specialist`](file:///D:/TadpoleOS-Dev/.agent/agents/tadpole-backend-specialist.md) | [`rust-pro`](file:///D:/TadpoleOS-Dev/.agent/skills/rust-pro/SKILL.md), [`tauri-pro`](file:///D:/TadpoleOS-Dev/.agent/skills/tauri-pro/SKILL.md), [`api-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/api-patterns/SKILL.md) |
| **React UI, Zustand & Tailwind** | [`tadpole-frontend-specialist`](file:///D:/TadpoleOS-Dev/.agent/agents/tadpole-frontend-specialist.md) | [`frontend-design`](file:///D:/TadpoleOS-Dev/.agent/skills/frontend-design/SKILL.md), [`tailwind-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/tailwind-patterns/SKILL.md), [`zustand-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/zustand-patterns/SKILL.md) |
| **System Audit & Blast Radius** | [`nexus-engineer`](file:///D:/TadpoleOS-Dev/.agent/agents/nexus-engineer.md) | [`architecture`](file:///D:/TadpoleOS-Dev/.agent/skills/architecture/SKILL.md), [`improve-codebase-architecture`](file:///D:/TadpoleOS-Dev/.agent/skills/improve-codebase-architecture/SKILL.md), [`code-review-graph`](file:///D:/TadpoleOS-Dev/.agent/skills/code-review-graph/SKILL.md) |
| **Multi-Agent Coordination** | [`orchestrator`](file:///D:/TadpoleOS-Dev/.agent/agents/orchestrator.md) | [`coordinator-mode`](file:///D:/TadpoleOS-Dev/.agent/skills/coordinator-mode/SKILL.md), [`parallel-agents`](file:///D:/TadpoleOS-Dev/.agent/skills/parallel-agents/SKILL.md), [`handoff`](file:///D:/TadpoleOS-Dev/.agent/skills/handoff/SKILL.md) |
| **Database & Vector Memory** | [`database-architect`](file:///D:/TadpoleOS-Dev/.agent/agents/database-architect.md) | [`database-design`](file:///D:/TadpoleOS-Dev/.agent/skills/database-design/SKILL.md), [`vector-memory-lancedb`](file:///D:/TadpoleOS-Dev/.agent/skills/vector-memory-lancedb/SKILL.md), [`domain-modeling`](file:///D:/TadpoleOS-Dev/.agent/skills/domain-modeling/SKILL.md) |
| **Defensive Security & Auth** | [`security-auditor`](file:///D:/TadpoleOS-Dev/.agent/agents/security-auditor.md) | [`vulnerability-scanner`](file:///D:/TadpoleOS-Dev/.agent/skills/vulnerability-scanner/SKILL.md), [`pii-redaction`](file:///D:/TadpoleOS-Dev/.agent/skills/pii-redaction/SKILL.md) |
| **Offensive Security & Red Team** | [`penetration-tester`](file:///D:/TadpoleOS-Dev/.agent/agents/penetration-tester.md) | [`red-team-tactics`](file:///D:/TadpoleOS-Dev/.agent/skills/red-team-tactics/SKILL.md), [`vulnerability-scanner`](file:///D:/TadpoleOS-Dev/.agent/skills/vulnerability-scanner/SKILL.md) |
| **Testing & Invariant Verification** | [`test-engineer`](file:///D:/TadpoleOS-Dev/.agent/agents/test-engineer.md) / [`qa-automation-engineer`](file:///D:/TadpoleOS-Dev/.agent/agents/qa-automation-engineer.md) | [`tdd-workflow`](file:///D:/TadpoleOS-Dev/.agent/skills/tdd-workflow/SKILL.md), [`testing-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/testing-patterns/SKILL.md), [`webapp-testing`](file:///D:/TadpoleOS-Dev/.agent/skills/webapp-testing/SKILL.md) |
| **Root Cause Debugging** | [`debugger`](file:///D:/TadpoleOS-Dev/.agent/agents/debugger.md) | [`systematic-debugging`](file:///D:/TadpoleOS-Dev/.agent/skills/systematic-debugging/SKILL.md), [`performance-profiling`](file:///D:/TadpoleOS-Dev/.agent/skills/performance-profiling/SKILL.md) |
| **Mobile Apps (Android / iOS)** | [`mobile-developer`](file:///D:/TadpoleOS-Dev/.agent/agents/mobile-developer.md) | [`mobile-design-android`](file:///D:/TadpoleOS-Dev/.agent/skills/mobile-design-android/SKILL.md), [`mobile-design-ios`](file:///D:/TadpoleOS-Dev/.agent/skills/mobile-design-ios/SKILL.md) |
| **Project Planning & Strategy** | [`project-planner`](file:///D:/TadpoleOS-Dev/.agent/agents/project-planner.md) / [`product-owner`](file:///D:/TadpoleOS-Dev/.agent/agents/product-owner.md) | [`plan-writing`](file:///D:/TadpoleOS-Dev/.agent/skills/plan-writing/SKILL.md), [`brainstorming`](file:///D:/TadpoleOS-Dev/.agent/skills/brainstorming/SKILL.md), [`architecture`](file:///D:/TadpoleOS-Dev/.agent/skills/architecture/SKILL.md) |
| **Customer Backend & Ingestion** | [`customer-backend-specialist`](file:///D:/TadpoleOS-Dev/.agent/agents/customer-backend-specialist.md) | [`api-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/api-patterns/SKILL.md), [`smb-data-ingestion`](file:///D:/TadpoleOS-Dev/.agent/skills/smb-data-ingestion/SKILL.md), [`digital-twin-voice`](file:///D:/TadpoleOS-Dev/.agent/skills/digital-twin-voice/SKILL.md) |

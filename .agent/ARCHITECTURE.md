> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Documentation / ARCHITECTURE
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[ARCHITECTURE]`)

# Antigravity Kit Architecture

> Comprehensive AI Agent Capability Expansion Toolkit

## 🧠 Core Intelligence & Routing

- **Intelligent Routing**: Located at `.agent/skills/intelligent-routing/SKILL.md`. This is the primary logic gate for agent selection and task routing.

---

## 📋 Overview

Antigravity Kit is a modular system consisting of:

- **20 Specialist Agents** - Role-based AI personas
- **48 Skills** - Domain-specific knowledge modules
- **16 Workflows** - Slash command procedures

---

## 🏗️ Directory Structure

```plaintext
.agent/
├── ARCHITECTURE.md          # This file
├── agents/                  # 20 Specialist Agents
├── skills/                  # 48 Skills
├── workflows/               # 16 Slash Commands
├── rules/                   # Global Rules
└── scripts/                 # Master Validation Scripts
```

---

## 🤖 Agents (28)

Specialist AI personas for different domains.

| Agent                    | Focus                      | Skills Used                                              |
| ------------------------ | -------------------------- | -------------------------------------------------------- |
| `orchestrator`           | Multi-agent coordination   | parallel-agents, behavioral-modes                        |
| `project-planner`        | Discovery, task planning   | brainstorming, plan-writing, architecture                |
| `frontend-specialist`    | Web UI/UX                  | frontend-design, react-best-practices, tailwind-patterns |
| `backend-specialist`     | Generic backend fallback   | api-patterns, nodejs-best-practices, database-design     |
| `tadpole-backend-specialist` | Tadpole Rust backend    | rust-pro, api-patterns, database-design                  |
| `customer-backend-specialist` | Customer backend safety | api-patterns, database-design, server-management         |
| `database-architect`     | Schema, SQL                | database-design, prisma-expert                           |
| `mobile-developer`       | iOS, Android, RN           | mobile-design                                            |
| `game-developer`         | Game logic, mechanics      | game-development                                         |
| `devops-engineer`        | CI/CD, Docker              | deployment-procedures, docker-expert                     |
| `security-auditor`       | Security compliance        | vulnerability-scanner, red-team-tactics                  |
| `penetration-tester`     | Offensive security         | red-team-tactics                                         |
| `test-engineer`          | Testing strategies         | testing-patterns, tdd-workflow, webapp-testing           |
| `debugger`               | Root cause analysis        | systematic-debugging                                     |
| `performance-optimizer`  | Speed, Web Vitals          | performance-profiling                                    |
| `seo-specialist`         | Ranking, visibility        | seo-fundamentals, geo-fundamentals                       |
| `documentation-writer`   | Manuals, docs              | documentation-templates                                  |
| `product-manager`        | Requirements, user stories | plan-writing, brainstorming                              |
| `product-owner`          | Strategy, backlog, MVP     | plan-writing, brainstorming                              |
| `qa-automation-engineer` | E2E testing, CI pipelines  | webapp-testing, testing-patterns                         |
| `code-archaeologist`     | Legacy code, refactoring   | clean-code, code-review-checklist                        |
| `explorer-agent`         | Codebase analysis          | -                                                        |
| `compliance-officer`     | Legal, privacy, licensing  | vulnerability-scanner, web-design-guidelines             |
| `growth-engineer`        | Analytics, funnels, experiments | seo-fundamentals, frontend-design                    |
| `nexus-engineer`         | System audit, blast radius | clean-code, rust-pro, vulnerability-scanner              |
| `release-manager`        | Release safety, rollout    | deployment-procedures, verify-changes                    |
| `sre-architect`          | Reliability, SLOs, incidents | server-management, performance-profiling               |
| `ux-designer`            | UX journeys, design systems | frontend-design, web-design-guidelines                   |

---

## 🧩 Skills (49)

Modular knowledge domains that agents can load on-demand based on task context.

### Frontend & UI

| Skill                   | Description                                                           |
| ----------------------- | --------------------------------------------------------------------- |
| `react-best-practices`  | React & Next.js performance optimization (Vercel - 57 rules)          |
| `web-design-guidelines` | Web UI audit - 100+ rules for accessibility, UX, performance (Vercel) |
| `tailwind-patterns`     | Tailwind CSS v4 utilities                                             |
| `frontend-design`       | UI/UX patterns, design systems                                        |

### Backend & API

| Skill                   | Description                                                        |
| ----------------------- | ------------------------------------------------------------------ |
| `api-patterns`          | REST, GraphQL, tRPC                                                |
| `nestjs-expert`         | NestJS modules, DI, decorators                                     |
| `nodejs-best-practices` | Node.js async, modules                                             |
| `python-patterns`       | Python standards, FastAPI                                          |
| `rust-pro`              | Master Rust 1.75+ with modern async patterns and type system      |

### Database

| Skill             | Description                 |
| ----------------- | --------------------------- |
| `database-design` | Schema design, optimization |
| `prisma-expert`   | Prisma ORM, migrations      |

### TypeScript/JavaScript

| Skill               | Description                         |
| ------------------- | ----------------------------------- |
| `typescript-expert` | Type-level programming, performance |

### Cloud & Infrastructure

| Skill                   | Description               |
| ----------------------- | ------------------------- |
| `docker-expert`         | Containerization, Compose |
| `deployment-procedures` | CI/CD, deploy workflows   |
| `server-management`     | Infrastructure management |

### Testing & Quality

| Skill                   | Description                                                    |
| ----------------------- | -------------------------------------------------------------- |
| `testing-patterns`      | Jest, Vitest, strategies                                       |
| `webapp-testing`        | E2E, Playwright                                                |
| `tdd-workflow`          | Test-driven development                                        |
| `code-review-checklist` | Code review standards                                          |
| `lint-and-validate`     | Linting, validation                                            |
| `code-review-graph`     | Token-efficient code review using Tree-sitter AST graphs/MCP   |
| `verify-changes`        | Prove code works by running it, not just checking it exists   |

### Security

| Skill                   | Description              |
| ----------------------- | ------------------------ |
| `vulnerability-scanner` | Security auditing, OWASP |
| `red-team-tactics`      | Offensive security       |

### Architecture & Planning

| Skill                 | Description                                                        |
| --------------------- | ------------------------------------------------------------------ |
| `app-builder`         | Full-stack app scaffolding                                         |
| `architecture`        | System design patterns                                             |
| `plan-writing`        | Task planning, breakdown                                           |
| `brainstorming`       | Socratic questioning                                               |
| `aletheia-reasoning`  | Structured iterative reasoning framework with explicit verification|

### Mobile

| Skill           | Description           |
| --------------- | --------------------- |
| `mobile-design` | Mobile UI/UX patterns |

### Game Development

| Skill              | Description           |
| ------------------ | --------------------- |
| `game-development` | Game logic, mechanics |

### SEO & Growth

| Skill              | Description                   |
| ------------------ | ----------------------------- |
| `seo-fundamentals` | SEO, E-E-A-T, Core Web Vitals |
| `geo-fundamentals` | GenAI optimization            |

### Shell/CLI

| Skill                | Description               |
| -------------------- | ------------------------- |
| `bash-linux`         | Linux commands, scripting |
| `powershell-windows` | Windows PowerShell        |

### Other

| Skill                     | Description                                                       |
| ------------------------- | ----------------------------------------------------------------- |
| `clean-code`              | Coding standards (Global)                                         |
| `behavioral-modes`        | Agent personas                                                    |
| `parallel-agents`         | Multi-agent patterns                                              |
| `mcp-builder`             | Model Context Protocol                                            |
| `documentation-templates` | Doc formats                                                       |
| `i18n-localization`       | Internationalization                                              |
| `performance-profiling`   | Web Vitals, optimization                                          |
| `systematic-debugging`    | Troubleshooting                                                   |
| `batch-operations`        | Apply operations across multiple files simultaneously             |
| `context-compression`     | Manage and compress conversation context in long sessions         |
| `coordinator-mode`        | High-order multi-agent orchestration and task mapping             |
| `explorer-scout`          | Scouting and analysis for GitHub repositories                    |
| `memory-system`           | Persistent cross-session memory management                        |
| `mission-analyst`         | Automated post-mission debriefing and memory recording            |
| `simplify-code`           | Reduce complexity of over-engineered code                         |
| `skillify`                | Auto-create new skills from repetitive workflows                  |

---

## 🔄 Workflows (20)

Slash command procedures. Invoke with `/command`.

| Command          | Description                                              |
| ---------------- | -------------------------------------------------------- |
| `/anneal`        | Self-annealing loop and protocol revisions               |
| `/audit`         | Compliance and security audits                           |
| `/brainstorm`    | Socratic discovery                                       |
| `/create`        | Create new features                                      |
| `/debug`         | Debug issues                                             |
| `/deploy`        | Deploy application                                       |
| `/enhance`       | Improve existing code                                    |
| `/github_scout`  | Scouting GitHub repositories                             |
| `/orchestrate`   | Multi-agent coordination                                 |
| `/plan`          | Task breakdown                                           |
| `/preview`       | Preview changes                                          |
| `/refactor`      | Decoupling and structural refactoring                    |
| `/report`        | Swarm metrics and performance reports                    |
| `/status`        | Check project status                                     |
| `/test`          | Run tests                                                |
| `/ui-ux-pro-max` | Design with 50 styles                                    |

---

## 🎯 Skill Loading Protocol

```plaintext
User Request → Skill Description Match → Load SKILL.md
                                            ↓
                                    Read references/
                                            ↓
                                    Read scripts/
```

### Skill Structure

```plaintext
skill-name/
├── SKILL.md           # (Required) Metadata & instructions
├── scripts/           # (Optional) Python/Bash scripts
├── references/        # (Optional) Templates, docs
└── assets/            # (Optional) Images, logos
```

### Enhanced Skills (with scripts/references)

| Skill               | Files | Coverage                            |
| ------------------- | ----- | ----------------------------------- |
| `ui-ux-pro-max`     | 27    | 50 styles, 21 palettes, 50 fonts    |
| `app-builder`       | 20    | Full-stack scaffolding              |

---

## 🛠️ Scripts (2)

Master validation scripts that orchestrate skill-level scripts.

### Master Scripts

| Script          | Purpose                                 | When to Use              |
| --------------- | --------------------------------------- | ------------------------ |
| `checklist.py`  | Priority-based validation (Core checks) | Development, pre-commit  |
| `verify_all.py` | Comprehensive verification (All checks) | Pre-deployment, releases |

### Usage

```bash
# Quick validation during development
python .agent/scripts/checklist.py .

# Full verification before deployment
python .agent/scripts/verify_all.py . --url http://localhost:3000
```

### What They Check

**checklist.py** (Core checks):

- Security (vulnerabilities, secrets)
- Code Quality (lint, types)
- Schema Validation
- Test Suite
- UX Audit
- SEO Check

**verify_all.py** (Full suite):

- Everything in checklist.py PLUS:
- Lighthouse (Core Web Vitals)
- Playwright E2E
- Bundle Analysis
- Mobile Audit
- i18n Check

For details, see [scripts/README.md](scripts/README.md)

---

## 📊 Statistics

| Metric              | Value                         |
| ------------------- | ----------------------------- |
| **Total Agents**    | 28                            |
| **Total Skills**    | 49                            |
| **Total Workflows** | 20                            |
| **Total Scripts**   | 2 (master) + 18 (skill-level) |
| **Coverage**        | ~90% web/mobile development   |

---

## 🔗 Quick Reference

| Need     | Agent                 | Skills                                |
| -------- | --------------------- | ------------------------------------- |
| Web App  | `frontend-specialist` | react-best-practices, frontend-design |
| API      | `tadpole-backend-specialist` | rust-pro, api-patterns              |
| Customer API | `customer-backend-specialist` | api-patterns, database-design       |
| Mobile   | `mobile-developer`    | mobile-design                         |
| Database | `database-architect`  | database-design, prisma-expert        |
| Security | `security-auditor`    | vulnerability-scanner                 |
| Testing  | `test-engineer`       | testing-patterns, webapp-testing      |
| Debug    | `debugger`            | systematic-debugging                  |
| Plan     | `project-planner`     | brainstorming, plan-writing           |
---
name: plan-writing
description: Structured task planning with clear breakdowns, dependencies, and verification criteria. Use when implementing features, refactoring, or any multi-step work.
when_to_use: "When creating structured task plans, breaking down features into tasks, or defining verification criteria. Use with /plan workflow."
allowed-tools: Read, Glob, Grep
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / plan-writing
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Structured Plan Writing Protocol

> **Purpose**: Convert complex requirements into actionable, atomic tasks with concrete verification gates.
> **Workflow Binding**: Used directly during the [`/plan`](../../workflows/plan.md) workflow.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** planning rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/implementation_plan_templates.md`](./references/implementation_plan_templates.md) | Feature blueprints, expand-contract slicing guides, verification checklists | Drafting architectural implementation plans |

---

## 📐 1. Task Breakdown Principles

1. **Keep It Short**: Aim for 5–10 clear, focused tasks maximum.
2. **Be Specific**: Specify exact file paths, line ranges, and function names (e.g. `src/services/agent_service.ts`).
3. **Atomic Verification**: Every single task must include an explicit verification step (`Verify: npm test`).
4. **Slicing Strategy**:
   - **Feature Work**: Vertical tracer bullets (schema ➔ backend route ➔ frontend state ➔ UI).
   - **Wide Refactors**: Expand-Contract slicing (Expand new ➔ Migrate callers ➔ Contract old).

---

## 🧭 2. Plan Structure Format

```markdown
# [Task Title]

## 1. Goal
One-sentence statement of what we are building or fixing.

## 2. Tasks
- [ ] Task 1: [Specific modification] ➔ Verify: [Test command / check]
- [ ] Task 2: [Specific modification] ➔ Verify: [Test command / check]

## 3. Final Verification Gate
- Automated tests: `npm run test && cargo test`
- Parity check: `python execution/parity_guard.py .`
```
---
name: plan-writing
description: "Structured task planning with clear breakdowns, dependencies, and verification criteria. Use when implementing features, refactoring, or any multi-step work."
when_to_use: "When creating structured task plans, breaking down complex features into atomic tasks, or defining verification criteria. Use with the /plan workflow."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / plan-writing
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Monolithic unbounded plans, omitted verification commands, or unscoped blast radius.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Structured Plan Writing Protocol

> **Purpose**: Convert complex architectural requirements into actionable, atomic tasks with concrete, measurable verification gates.  
> **Workflow Bindings**: Integrated into [`/plan`](file:///D:/TadpoleOS-Dev/.agent/workflows/plan.md), [`/create`](file:///D:/TadpoleOS-Dev/.agent/workflows/create.md), and [`/refactor`](file:///D:/TadpoleOS-Dev/.agent/workflows/refactor.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/implementation_plan_templates.md`](file:///D:/TadpoleOS-Dev/.agent/skills/plan-writing/references/implementation_plan_templates.md) | Feature blueprints, expand-contract slicing guides, verification checklists | Drafting architectural implementation plans |
| [`scripts/plan_linter.py`](file:///D:/TadpoleOS-Dev/.agent/skills/plan-writing/scripts/plan_linter.py) | Automated plan linter verifying goals, task atomicity, and verification commands | Validating plan markdown structure before execution |

---

## 📐 1. Task Breakdown Principles

1. **Keep It Short**: Aim for 5–10 clear, focused tasks maximum per plan.
2. **Be Specific**: Specify exact file paths, line ranges, and function names (e.g. `src/services/agent_service.ts`).
3. **Atomic Verification**: Every single task must include an explicit verification step (`➔ Verify: npm test`).
4. **Slicing Strategy**:
   - **Feature Work**: Vertical tracer bullets (schema ➔ backend route ➔ frontend state ➔ UI).
   - **Wide Refactors**: Expand-Contract slicing (Expand new ➔ Migrate callers ➔ Contract old).

---

## 🧭 2. Canonical Plan Structure Format

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

---

## 🧠 3. Socratic Gate for Implementation Plans

Before presenting or executing a plan, verify:
- **Gate 1 (Atomicity)**: Can each task be implemented and independently verified in $<15$ minutes?
- **Gate 2 (Verification Command)**: Does every single `- [ ]` task list an executable verification command?
- **Gate 3 (Blast Radius Guard)**: Has `npm run graph:blast:guard -- --path <file>` been run to confirm dependent callers?

---

## 🚫 4. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Vague Tasks** | "Fix bugs in frontend" gives no guidance. | State exact filenames and functions to touch. |
| **Missing Verification** | Assuming edits work without running checks. | Append explicit `➔ Verify: <command>` to every task. |
| **Plan Bloat (>15 Tasks)** | Cognitive overload and context degradation. | Split into phase 1 and phase 2 sub-plans. |
| **Big-Bang Refactors** | Breaking all callers simultaneously. | Use Expand-Contract pattern with backwards-compatible aliases. |

---

## 🛠️ 5. Deterministic Verification Gate

Validate plan format and task checklist structure:

```powershell
# 1. Run built-in self-tests
python .agent/skills/plan-writing/scripts/plan_linter.py --test

# 2. Lint a markdown plan file
python .agent/skills/plan-writing/scripts/plan_linter.py --file path/to/plan.md
```
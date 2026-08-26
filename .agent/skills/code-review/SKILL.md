---
name: code-review
description: Two-axis parallel code review comparing HEAD against a fixed point. Axis 1 (Standards) checks repo conventions and Fowler code smells. Axis 2 (Spec) checks line-by-line requirements coverage.
when_to_use: "Use when reviewing a branch, PR, diff, or work-in-progress against a fixed commit SHA, branch, or merge-base."
allowed-tools: Read, Glob, Grep, Bash
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / code-review
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Two-Axis Code Review Protocol

Review the diff between `HEAD` and a fixed point along two distinct, un-merged axes:
- **Standards Axis**: Does the code conform to documented repo standards and Fowler code smells?
- **Spec Axis**: Does the code faithfully implement the originating issue / PRD / spec?

Both axes run as **parallel sub-agents** so they do not pollute each other's context.

---

## 1. Pin the Fixed Point

1. Determine the fixed point (commit SHA, branch, tag, `main`, `HEAD~5`).
2. Verify resolution: `git rev-parse <fixed-point>`.
3. Capture diff once: `git diff <fixed-point>...HEAD` (three-dot comparison against merge-base).
4. Capture commit log: `git log <fixed-point>..HEAD --oneline`.

---

## 2. Identify Sources

### Spec Source
Look for the spec in this order:
1. Issue references in commit messages (`#123`, `Closes #45`).
2. A spec/PRD file path passed in conversation or found under `docs/`, `specs/`, or `.tmp/`.
3. If no spec is found, note "No spec available" and skip the Spec sub-agent.

### Standards Source & Fowler Smell Baseline
Combine repo documentation (`CODING_STANDARDS.md`, `CONTRIBUTING.md`, `AGENTS.md`) with the **Fowler Code Smell Baseline**:

- **Mysterious Name**: Variable, function, or type name doesn't reveal intent $\rightarrow$ Rename.
- **Duplicated Code**: Identical logic shape in multiple hunks/files $\rightarrow$ Extract shared helper.
- **Feature Envy**: Method reaching into another object's data more than its own $\rightarrow$ Move method to data owner.
- **Data Clumps**: Same group of parameters traveling together $\rightarrow$ Introduce dedicated type.
- **Primitive Obsession**: Primitive type standing in for a domain concept $\rightarrow$ Create domain type.
- **Repeated Switches**: Duplicate `switch`/`if` cascades on same type $\rightarrow$ Polymorphism or lookup table.
- **Shotgun Surgery**: Single logical change requires edits across many files $\rightarrow$ Consolidate module.
- **Divergent Change**: Single module edited for multiple unrelated reasons $\rightarrow$ Split module.
- **Speculative Generality**: Abstraction added for hypothetical future needs $\rightarrow$ Inline back.
- **Message Chains**: Long `a.b().c().d()` navigation $\rightarrow$ Encapsulate walk behind method on first object.
- **Middle Man**: Class/function mostly delegating onward $\rightarrow$ Call target directly.
- **Refused Bequest**: Subclass overriding/ignoring inherited behavior $\rightarrow$ Use composition over inheritance.

> **Rule**: Documented repo standards always override baseline smells.

---

## 3. Execute Parallel Sub-Agents

Spawn two parallel sub-agents (`subagent_type=General`):

1. **Standards Sub-Agent**: Evaluates diff strictly against repo standards and the Fowler smell baseline.
2. **Spec Sub-Agent**: Compares diff against the spec for missing requirements, scope creep, or misimplementations.

---

## 4. Aggregate Report

Present findings separately under `## Standards Findings` and `## Spec Findings`.

> [!IMPORTANT]
> **Separation Rule**: Never merge or rerank findings across axes. Code can pass Standards while failing Spec (wrong feature written cleanly), or pass Spec while failing Standards (correct feature written messily).

End with a 2-line summary:
- **Standards Status**: [PASS/FAIL] - Total findings and worst issue.
- **Spec Status**: [PASS/FAIL] - Total findings and worst issue.
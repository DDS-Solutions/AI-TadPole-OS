---
name: code-review
description: "Two-axis parallel code review comparing HEAD against a fixed point. Axis 1 (Standards) audits repo conventions and Fowler smells; Axis 2 (Spec) verifies line-by-line requirements coverage."
when_to_use: "Use when reviewing a pull request, diff, or branch against a base commit, target branch, or specification contract."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / code-review
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Merged-axis bias, unverified requirement coverage, ungrounded review comments.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Two-Axis Code Review Protocol

Review the diff between `HEAD` and a fixed point along two distinct, un-merged axes:
- **Standards Axis**: Does the code conform to documented repo standards, clean-code invariants, and Fowler code smells?
- **Spec Axis**: Does the code faithfully implement the originating issue, blueprint, or PRD requirements?

Both axes run as **parallel sub-agents** so that cognitive bias from one axis does not pollute the other.

---

## 📚 Progressive Disclosure & L3 Reference Index

Consult deep **L3 Reference Guides** and verification tools on demand:
- **Fowler Smells Catalog & Two-Axis Architecture**: [fowler_code_smells_and_two_axis_review.md](file:///D:/TadpoleOS-Dev/.agent/skills/code-review/references/fowler_code_smells_and_two_axis_review.md)
- **Deterministic Two-Axis Reviewer**: [two_axis_reviewer.py](file:///D:/TadpoleOS-Dev/.agent/skills/code-review/scripts/two_axis_reviewer.py)

---

## 1. Pin the Fixed Point

Before initiating review, establish a deterministic baseline:
1. Determine the fixed point (commit SHA, branch, tag, `main`, `HEAD~5`).
2. Verify resolution: `git rev-parse <fixed-point>`.
3. Capture diff once: `git diff <fixed-point>...HEAD` (three-dot comparison against merge-base).
4. Capture commit log: `git log <fixed-point>..HEAD --oneline`.

---

## 2. Identify Review Sources

### Spec Source
Locate the originating requirements in priority order:
1. Issue references in commit messages (`#123`, `Closes #45`).
2. A blueprint, spec, or PRD file passed in conversation or found under `docs/`, `specs/`, or `.tmp/`.
3. If no spec is found, note "No spec available" and skip the Spec sub-agent.

### Standards Source & Fowler Smell Baseline
Combine repository documentation (`CODING_STANDARDS.md`, `CONTRIBUTING.md`, `AGENTS.md`) with the **Fowler Code Smell Baseline**:
- **Mysterious Name**: Variable, function, or type name does not reveal intent $\rightarrow$ Rename.
- **Duplicated Code**: Identical logic shapes in multiple hunks/files $\rightarrow$ Extract shared helper.
- **Feature Envy**: Function accessing another struct's data more than its own $\rightarrow$ Move logic to owner.
- **Data Clumps**: Same group of parameters traveling together $\rightarrow$ Introduce dedicated type.
- **Primitive Obsession**: Primitive type standing in for a domain concept $\rightarrow$ Create domain type.
- **Repeated Switches**: Duplicate `switch`/`match` cascades on same type $\rightarrow$ Polymorphism or lookup table.
- **Shotgun Surgery**: Single change requires edits across many files $\rightarrow$ Consolidate module.
- **Divergent Change**: Single module edited for multiple unrelated reasons $\rightarrow$ Split module (SRP).
- **Speculative Generality**: Abstraction added for hypothetical future needs $\rightarrow$ Inline back (YAGNI).
- **Message Chains**: Long navigation chains $\rightarrow$ Encapsulate walk behind root method.
- **Middle Man**: Class/function mostly delegating onward $\rightarrow$ Call target directly.
- **Refused Bequest**: Subclass overriding inherited behavior with no-ops $\rightarrow$ Composition.

> **Rule**: Documented repository standards and executable invariants always supersede baseline smells.

---

## 3. Execute Parallel Sub-Agents

Spawn two parallel sub-agents (`subagent_type=General`):
1. **Standards Sub-Agent**: Evaluates the diff strictly against repository conventions, clean-code rules, and Fowler smells.
2. **Spec Sub-Agent**: Compares diff against the specification for missing requirements, scope creep, or misimplementations.

---

## 4. Aggregate Report & Separation Rule

Present findings separately under `## Axis 1: Standards Findings` and `## Axis 2: Spec Findings`.

> [!IMPORTANT]
> **Separation Rule**: Never merge or rerank findings across axes. Code can pass Standards while failing Spec (wrong feature written cleanly), or pass Spec while failing Standards (correct feature written messily).

### Mandatory Summary Format
Conclude every review with the two-line summary:
- **Standards Status**: [PASS/FAIL] - Total findings and worst issue.
- **Spec Status**: [PASS/FAIL] - Total findings and worst issue.

---

## 🛠️ 5. Deterministic Verification Gate

Validate Two-Axis review reports and diff compliance:

```powershell
# Run built-in self tests
python .agent/skills/code-review/scripts/two_axis_reviewer.py --test

# Audit a generated review report for Two-Axis contract compliance
python .agent/skills/code-review/scripts/two_axis_reviewer.py --file reports/review_pr_123.md
```

[//]: # (Metadata: [CODE_REVIEW])
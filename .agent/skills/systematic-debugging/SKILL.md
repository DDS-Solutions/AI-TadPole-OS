---
name: systematic-debugging
description: "4-phase systematic debugging methodology with root cause analysis, 3-fix circuit breaker, and evidence-based verification. Use when debugging complex issues."
when_to_use: "When debugging complex issues, performing root cause analysis, isolating race conditions, or resolving intermittent failures. Use with the /debug workflow."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.2.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / systematic-debugging
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Shotgun debugging, speculative guesswork, unverified fixes, or infinite repair loops.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Systematic Debugging & RCA Protocol

> **Philosophy**: Never touch code or propose speculative fixes until a tight, red-capable reproduction test proves the defect. Eliminate guesswork through binary hypothesis discrimination and 5-Whys root cause analysis.  
> **Skill Synergy & Boundaries**:
> - **Regression Tests & TDD**: Delegates test authoring to [`tdd-workflow`](../tdd-workflow/SKILL.md).
> - **Graph Blast Radius**: Delegates callsite and caller analysis to [`code-review-graph`](../code-review-graph/SKILL.md).
> - **Workflow Bindings**: Integrated directly into [`/debug`](../../workflows/debug.md) and [`/anneal`](../../workflows/anneal.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign debugging rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/rca_and_hypothesis_testing.md`](./references/rca_and_hypothesis_testing.md) | Hypothesis discrimination tables, 3-Fix circuit breaker state machine, 5-Whys root cause chains, multi-layer bisection schema | Isolating elusive bugs & architecting regression tests |
| [`scripts/reproduction_runner.py`](./scripts/reproduction_runner.py) | Automated reproduction runner measuring execution latency and testing for command flakiness | Establishing tight (<2s) reproduction loops |

---

## 🔄 1. The 4-Phase Debugging Process

```
[PHASE 1: REPRODUCE] ───> [PHASE 2: ISOLATE] ───> [PHASE 3: ROOT CAUSE] ───> [PHASE 4: VERIFY]
 Build minimal, fast      Formulate competing    Find point of divergence     Prove green pass &
 red failing test (<2s)   hypotheses ($H_A/H_B$) via telemetry map           add permanent test
```

### Phase 1: Reproduce & Tighten Feedback Loop
- 🛑 **STRICT GATE**: Do NOT modify code or hypothesize fixes until a **tight, red-capable reproduction command** is built and verified.
- **Criteria**:
  - **Red-Capable**: Fails on the user's exact symptom before any fix is applied.
  - **Deterministic**: 100% reproducible (use `reproduction_runner.py --flaky-check` to measure stability).
  - **Fast**: Completes in $< 2$ seconds.

### Phase 2: Isolate via Hypothesis Discrimination & Bisection
- Divide the system across its physical boundaries (React UI $\leftrightarrow$ Tauri IPC $\leftrightarrow$ Axum Engine $\leftrightarrow$ SQLite/LanceDB).
- Formulate two mutually exclusive hypotheses ($H_A$ vs $H_B$) with predicted observable evidence.
- Execute a non-destructive active probe to falsify one candidate.

### Phase 3: Identify the Point of Divergence
- Trace backwards from the symptom to the precise line where runtime state diverged from specification.
- Use `docs/ERROR_REGISTRY.json` and `docs/TELEMETRY_MAP.json` to verify log emitter locations.

### Phase 4: Verify & Anneal
- Apply minimal, targeted fix.
- Re-run reproduction command to prove green pass.
- Commit a permanent regression test witness to prevent recurrence.

---

## 🛑 2. The 3-Fix Circuit Breaker Rule

> **Circuit Breaker Rule**: If an agent attempts **3 consecutive fixes** that all fail to resolve the defect:
> 1. **HALT IMMEDIATELY**: Do not make a 4th speculative edit.
> 2. **REVERT SCRATCH EDITS**: Restore the working tree to the baseline state.
> 3. **ESCALATE AS LOGIC-BLOCKER**: Present the 3 falsified hypotheses, reproduction evidence, and request architectural alignment from Entity 0.

---

## 🧠 3. Socratic Gate for Bug Fixes

Before claiming an issue is resolved, verify:
- **Gate 1 (Red Before Green)**: Did the reproduction command verifiably fail (exit code $\ne 0$) prior to modifying the code?
- **Gate 2 (Circuit Breaker Guard)**: Are we within the 3-attempt limit without recursive patch thrashing?
- **Gate 3 (Root Cause vs Symptom)**: Does this fix resolve the underlying flaw rather than suppressing the symptom with a fallback or `@ts-ignore`?
- **Gate 4 (No Regressions)**: Did all workspace test suites pass (`npm test`, `cargo check`) after applying the fix?

---

## 🚫 4. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Shotgun Debugging** | Randomly changing lines of code hoping the error disappears. | Halt immediately; build a minimal failing test first. |
| **Symptom Masking** | Wrapping failing calls in `try {} catch {}` or adding default values. | Trace back to the origin of invalid state and fix the root cause. |
| **Infinite Fix Thrashing** | Attempting 5+ blind variations after previous attempts failed. | Trigger 3-Fix Circuit Breaker; halt and review architecture. |
| **Flaky Repro Loops** | Using reproduction commands that take 2 minutes or fail randomly. | Isolate into a minimal unit test executing in $< 2$ seconds. |
| **Fix Without Regression Test** | The same bug reappears in a future refactor. | Commit an automated test witness to `tests/`. |

---

## 🛠️ 5. Deterministic Verification Gate

Run reproduction commands and measure flakiness:

```powershell
# 1. Run built-in self-tests
python .agent/skills/systematic-debugging/scripts/reproduction_runner.py --test

# 2. Check stability and flakiness of reproduction command
python .agent/skills/systematic-debugging/scripts/reproduction_runner.py --flaky-check "npx vitest run tests/failing.test.ts"
```
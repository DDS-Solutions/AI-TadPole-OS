---
name: systematic-debugging
description: 4-phase systematic debugging methodology with root cause analysis and evidence-based verification. Use when debugging complex issues.
when_to_use: "When debugging complex issues, performing root cause analysis, or using evidence-based problem solving. Use with /debug workflow."
allowed-tools: Read, Glob, Grep
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / systematic-debugging
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Systematic Debugging

> Source: obra/superpowers

## Overview
This skill provides a structured approach to debugging that prevents random guessing and ensures problems are properly understood before solving.

## 4-Phase Debugging Process

### Phase 1: Reproduce & Tighten Feedback Loop

> 🛑 **STRICT GATE**: Do NOT touch code or hypothesize fixes until a **tight, red-capable reproduction command** is built and verified.

Build the tightest possible feedback loop in roughly this order:
1. **Failing test** (unit, integration, e2e) asserting the exact symptom.
2. **Curl / HTTP script** against a local endpoint.
3. **CLI invocation** diffing stdout/stderr against expected fixture.
4. **Captured trace / payload replay** through isolated code path.

#### Feedback Loop Quality Criteria
- [ ] **Red-Capable**: Drives the actual bug code path and fails on the user's *exact* symptom (not just "didn't crash").
- [ ] **Deterministic**: Returns identical pass/fail verdict on 100% of runs (for flaky bugs, loop 100x to raise repro rate).
- [ ] **Fast**: Completes in < 2 seconds.

```markdown
## Reproduction Gate Sign-off
- Command Executed: `<exact command line>`
- Output Log: `<pasted red failure log>`
- Loop Duration: `<seconds>`
```

### Phase 2: Isolate & Discriminative Probe
Narrow down the source using evidence and active hypothesis discrimination (Schema Harness protocol).

```markdown
## Isolation & Discriminative Probing
- When did this start happening?
- What changed recently?
- Does it happen in all environments?

### Hypothesis Discrimination (Probing Protocol)
Formulate two competing hypotheses and design an active probe:
- **Hypothesis A (H_A)**: [Proposed cause A] -> Expected probe outcome: [Outcome A]
- **Hypothesis B (H_B)**: [Proposed cause B] -> Expected probe outcome: [Outcome B]
- **Discriminative Probe Execution**: Execute non-destructive test to falsify one hypothesis.
```

### Phase 3: Understand
Find the root cause, not just symptoms.

```markdown
## Root Cause Analysis
### The 5 Whys
1. Why: [First observation]
2. Why: [Deeper reason]
3. Why: [Still deeper]
4. Why: [Getting closer]
5. Why: [Root cause]
```

### Phase 4: Fix & Verify
Fix and verify it's truly fixed.

```markdown
## Fix Verification
- [ ] Bug no longer reproduces
- [ ] Related functionality still works
- [ ] No new issues introduced
- [ ] Test added to prevent regression
```

## Debugging Checklist

```markdown
## Before Starting
- [ ] Can reproduce consistently
- [ ] Have minimal reproduction case
- [ ] Understand expected behavior

## During Investigation
- [ ] Check recent changes (git log)
- [ ] Check logs for errors
- [ ] Add logging if needed
- [ ] Use debugger/breakpoints

## After Fix
- [ ] Root cause documented
- [ ] Fix verified
- [ ] Regression test added
- [ ] Similar code checked
```

## Common Debugging Commands

```bash
# Recent changes
git log --oneline -20
git diff HEAD~5

# Search for pattern
grep -r "errorPattern" --include="*.ts"

# Check logs
pm2 logs app-name --err --lines 100
```

## Anti-Patterns

❌ **Random changes** - "Maybe if I change this..."
❌ **Ignoring evidence** - "That can't be the cause"
❌ **Assuming** - "It must be X" without proof
❌ **Not reproducing first** - Fixing blindly
❌ **Stopping at symptoms** - Not finding root cause
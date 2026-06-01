> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[debug]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
description: Forensic investigation command. Activates DEBUG mode for systematic root-cause analysis of implementation errors and sovereign logic faults.
---

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[debug]` in audit logs.
>
> ### 🤖 AI Persona Directive
> When `/debug` is active, operate as a **Forensic Systems Engineer**. Prioritize evidence over assumptions. Your goal is to identify the "Point of Divergence"—the exact moment the system state deviated from the intended Sovereign logic.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# /debug - Systematic Forensic Investigation

**Usage**: `/debug [issue_description/error_log]`

---

## 🎯 Purpose
This command activates **DEBUG** mode. It is used to resolve discrepancies in system behavior, recover from panics, and identify whether a failure is an **Implementation Bug** (Code) or a **Sovereign Logic Fault** (Directive).

---

## ⚙️ Forensic Behavior

When `/debug` is triggered, the AI must follow this rigorous investigative pipeline:

### 1. Telemetry Gathering (Evidence Collection)
Collect the "Digital Fingerprint" of the failure:
- **Runtime Logs**: Search `src/server-rs` for Tokio runtime panics or Axum middleware errors.
- **State Snapshot**: Review `tadpole.db` audit logs for the specific `mission_id`.
- **Environment Check**: Confirm OS specifics and `.env` configuration parity.
- **Frontend State**: Inspect Zustand store dumps or React console errors.

### 2. Component Isolation (The 4-Pillars)
Determine which pillar of the Sovereign infrastructure collapsed:
- **Gateway**: WebSocket/API protocol failures or routing mismatches.
- **Runner**: Intelligence Loop timeouts, Provider API failures, or LLM hallucinations.
- **Registry**: Persistence failures, SQLite migration mismatches, or schema drift.
- **Security**: `Budget Guard` or `Shield Layer` false positives/negatives.

### 3. Blast Radius Analysis (Graph Intelligence)
Before forming a hypothesis, use the symbol graph to understand the dependency chain.
- **Action**: Run `npm run graph:blast -- --path [affected_file]` to see every single caller and callee impacted by the failure.
- **Goal**: Prevent "Whack-a-Mole" debugging where a fix in one area breaks a dependent module.

### 4. The Divergence Test (Code vs. Directive)
Determine the nature of the fault:
- **Implementation Bug**: The code failed to execute the directive. $\rightarrow$ **Fix: Code Patch.**
- **Sovereign Logic Fault**: The code executed the directive perfectly, but the result was "Bad Output." $\rightarrow$ **Fix: Trigger [/anneal](anneal.md) workflow.**

---

## 📊 Output Format

```markdown
## 🔍 Forensic Report: [Issue]

### 1. Symptom & Evidence
- **Observation**: [What happened]
- **Evidence**: `[Error Message / Log Trace]`
- **Location**: `[File] : [Line]`

### 2. Hypothesis & Validation
**Hypothesis 1**: [Likely cause]
- **Test**: [What was checked] $\rightarrow$ **Result**: [Pass/Fail]

**Hypothesis 2**: [Alternative cause]
- **Test**: [What was checked] $\rightarrow$ **Result**: [Pass/Fail]

### 3. Root Cause Analysis (RCA)
🎯 **Point of Divergence**: [Detailed explanation of why the system failed.]
**Fault Type**: [Implementation Bug | Sovereign Logic Fault]

### 4. Remediation
```[language]
// 🔴 BEFORE (Faulty State)
[broken code or directive]

// 🟢 AFTER (Hardened State)
[fixed code or updated directive]

---

## 🚫 Key Principles

- **Evidence First**: Do not propose a fix until a hypothesis has been validated with telemetry or a test case.
- **Surgical Precision**: Fix the root cause, not the symptom. Use `graph:blast` to ensure no regressions.
- **No Silent Fixes**: Every `/debug` session must conclude with a "Prevention" step to ensure the system evolves.
- **Divergence Awareness**: Always ask: *"Did the code fail the directive, or did the directive fail the mission?"*

[//]: # (Metadata: [debug])

[//]: # (Metadata: [debug])

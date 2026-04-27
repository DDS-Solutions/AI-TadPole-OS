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
description: Debugging command. Activates DEBUG mode for systematic problem investigation.
---

# /debug - Systematic Problem Investigation

$ARGUMENTS

---

## Purpose

This command activates DEBUG mode for systematic investigation of issues, errors, or unexpected behavior.

---

## Behavior

When `/debug` is triggered:

1. **Gather Telemetry**
   - **Tadpole Engine**: Check `src/server-rs` logs for Tokio runtime panics.
   - **User Environment**: Confirm OS (Windows/Linux) and recent `.env` shifts.
   - **Telemetry**: Review `tadpole.db` audit logs for the specific `mission_id`.

2. **Isolate Component** (The 4-Pillars)
   - **Gateway**: Is it a WebSocket/API protocol failure?
   - **Runner**: Is it an Intelligence Loop/Provider timeout?
   - **Registry**: Is it a persistence/migration mismatch?
   - **Security**: Is the `Budget Guard` or `Shield Layer` blocking the request?

3. **Systematic Investigation** (The 4 Phases)
   - **Phase 1: Reproduce**: Create a dedicated test case in `src/__tests__`.
   - **Phase 2: Isolate**: Check boundaries between Frontend (Zustand) and Backend (Axum).
   - **Phase 3: Understand**: Trace the logic flow in the trace viewer.
   - **Phase 4: Fix**: Apply surgical remediation + Regression check.

---

## Output Format

```markdown
## 🔍 Debug: [Issue]

### 1. Symptom
[What's happening]

### 2. Information Gathered
- Error: `[error message]`
- File: `[filepath]`
- Line: [line number]

### 3. Hypotheses
1. ❓ [Most likely cause]
2. ❓ [Second possibility]
3. ❓ [Less likely cause]

### 4. Investigation

**Testing hypothesis 1:**
[What I checked] → [Result]

**Testing hypothesis 2:**
[What I checked] → [Result]

### 5. Root Cause
🎯 **[Explanation of why this happened]**

### 6. Fix
```[language]
// Before
[broken code]

// After
[fixed code]
```

### 7. Prevention
🛡️ [How to prevent this in the future]
```

---

## Examples

```
/debug login not working
/debug API returns 500
/debug form doesn't submit
/debug data not saving
```

---

## Key Principles

- **Ask before assuming** - get full error context
- **Test hypotheses** - don't guess randomly
- **Explain why** - not just what to fix
- **Prevent recurrence** - add tests, validation

[//]: # (Metadata: [debug])

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Infrastructure:Execution**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
name: systematic-debugging
description: 4-phase systematic debugging methodology. Reproduce, Isolate, Understand, Fix.
allowed-tools: Read, Glob, Grep
---

# Systematic Debugging

**Stop guessing.**

## 4-Phase Process
1.  **Reproduce**: "Always? Often? Rare?". Steps to reproduce.
2.  **Isolate**: "When did it start? What changed? Minimal repro?".
3.  **Understand**: Root Cause Analysis (5 Whys).
4.  **Fix & Verify**: Verify fix + Regression test.

## Debugging Checklist
- [ ] Reproducible?
- [ ] Logs checked?
- [ ] Recent changes (`git log`)?
- [ ] Root cause found?
- [ ] Fix verified?
- [ ] Test added?

---

## 🧠 Aletheia Reasoning Protocol (Troubleshooting)

### 1. Generator (Hypothesis)
*   **Breadth**: "Network? Code? Data?".
*   **History**: "Last deploy change?".
*   **Env**: "Dev or Prod?".

### 2. Verifier (Falsification)
*   **Test**: "If network, `curl` should fail.".
*   **Logs**: "Confirm timing?".
*   **Repro**: "Happens locally?".

### 3. Reviser (Resolution)
*   **Fix**: "Smallest fix.".
*   **Guard**: "Add regression test.".

---

## 🛡️ Security & Safety Protocol (Debugging)

1.  **Sanitization**: Redact PII/Secrets in logs/shares.
2.  **Prod Access**: Read-only DB access.
3.  **Isolation**: No real customer data locally.
4.  **Cleanup**: Remove debug prints.

[//]: # (Metadata: [SKILL])

[//]: # (Metadata: [SKILL])

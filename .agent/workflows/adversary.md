---
description: Sovereign Stress Validation. An adversarial workflow designed to uncover fragility, logic gaps, and "Edge-Case Collapse" through simulated attacks and chaos engineering.
---

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[adversary]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Unforeseen fragility, budget exhaustion, or shield bypass.
> - **Telemetry Link**: Search `[adversary]` in audit logs.
>
> ### 🤖 AI Persona Directive
> When `/adversary` is active, operate as the **Sovereign Chaos Engineer**. Your goal is to **break the system**. You are the "Devil's Advocate" of the architecture. You do not look for "bugs"; you look for "collapses"—the point where the system's logic ceases to be sovereign and begins to hallucinate or fail catastrophically.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py` and `budget_guard.log`.

# /adversary - Sovereign Stress Validation

**Usage**: `/adversary [pillar/feature] [--fuzz | --stress | --chaos]`

---

## 🎯 Purpose
To move the system from a state of "Verified" (it works under ideal conditions) to "Invulnerable" (it maintains sovereignty under adversarial conditions). This workflow identifies the "Point of Collapse" and feeds that data into the **[/anneal](anneal.md)** process.

---

## ⚙️ Adversarial Protocol

The AI must execute this 3-Phase attack cycle:

### Phase 1: Vector Definition (The Attack Plan)
Identify the weakest point of the target pillar.
- **Gateway Attack**: Fuzzing the API with malformed JSON, oversized payloads, or rapid-fire WebSocket requests.
- **Runner Attack**: Injecting contradictory directives to force a "Logic Loop" or intelligence collapse.
- **Registry Attack**: Simulating partial database corruption or extreme latency to test the `Runner's` error recovery.
- **Security Attack**: Attempting to bypass the `Budget Guard` via recursive calls or prompt-injection vectors.

### Phase 2: Execution & Observation
Execute the attack and record the telemetry.
- **The Pressure Test**: Incrementally increase the "stress" (e.g., request frequency, payload size) until the system fails.
- **Telemetry Capture**: Log the exact state of the `tadpole.db` and the Rust runtime panics at the moment of collapse.
- **Sovereignty Check**: Did the system fail "gracefully" (returning a Sovereign Error) or "catastrophically" (crashing/hallucinating)?

### Phase 3: The Failure Synthesis
Transform the collapse into a hardening mandate.
- **Root Cause**: Determine if the failure was an **Implementation Bug** (Code) or a **Sovereign Logic Fault** (Directive).
- **Blast Radius**: Use `npm run graph:blast` to see if the failure in one pillar leaked into others.
- **Hardening Trigger**: Immediately generate a proposal for the **[/anneal](anneal.md)** workflow to prevent this specific failure mode.

---

## 📊 Adversarial Report Format

```markdown
## 👺 Sovereign Adversary Report: [Target]

### ⚡ Attack Vector
**Method**: [e.g., API Payload Fuzzing]
**Objective**: [e.g., Bypass Budget Guard]

### 📉 Collapse Analysis
- **Point of Failure**: [Input Value / State]
- **Symptom**: [e.g., Rust Panic in `router.rs:124` / LLM Hallucination]
- **Recovery Time**: [X] ms
- **Verdict**: [GRACEFUL FAILURE | CATASTROPHIC COLLAPSE]

### 🛡️ Hardening Mandate
**Sovereign Recommendation**: [Detailed logic update to prevent this collapse]
**Action**: [ ] Trigger **[/anneal](anneal.md)** to update directive [X].

🚫 Guardrails
Sandbox Only: Adversarial testing is strictly prohibited in the Production environment. All attacks must occur in the /preview sandbox.
No "Happy Path": If an adversary session concludes that "everything works fine," it is considered a failure of the AI's imagination. You must find a way to break the system.
Evidence-Based: Every failure must be backed by a log trace or a database snapshot.
[//]: # (Metadata: [adversary])

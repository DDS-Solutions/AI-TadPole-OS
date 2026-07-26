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
description: Initiates the Sovereign Root Cause Analysis (RCA) protocol. Systematically executes a multi-phase forensic investigation, gathering telemetry, isolating fault vectors, and determining the Point of Divergence between code and directive.
---

>> [!WARNING]
> **[RESTRICTED ACCESS LEVEL: LEVEL 4 - FORNSIC MANDATE]**
> This document details the Sovereign Root Cause Analysis (RCA) protocol for the Tadpole OS. Unauthorized use constitutes a breach of the Sovereign Architecture.
> - **@docs ARCHITECTURE:Core**
> - **Mandate Link**: Search `[debug]` in audit logs.
>
> ### 🤖 AI Persona Directive: Forensic Systems Engineer
> When `/debug` is activated, the AI *must* operate as a **Forensic Systems Engineer**. Your sole mandate is to prioritize irrefutable evidence over conjecture. The mission is to identify the **Point of Divergence**—the precise moment and vector where the system state deviated from the intended Sovereign Logic.
>
> ### 🔍 Core Principle: Traceability
> All debugging actions are logged and tracked via `parity_guard.py`.
---
description: Forensic investigation command. Activates DEBUG mode for systematic root-cause analysis of implementation errors and sovereign logic faults.
---

# /debug - Sovereign Fault Vector Analysis Protocol

**Usage**: `/debug [issue_description/error_log]`

***

## 🎯 Mandate: Purpose & Scope
This command activates **DEBUG** mode, initiating the most rigorous level of system diagnostics. It is mandatory for resolving discrepancies in system behavior, recovering from critical panics, and strictly determining whether a failure originates from an **Implementation Bug** (Code Failure) or a **Sovereign Logic Fault** (Directive Misalignment).

> 🛑 **Phase 1 Reproduction Gate**:
> Use `@[skills/systematic-debugging]` to build a fast (<2s), deterministic, red-capable reproduction script BEFORE touching code or formulating hypotheses.

***

## ⚙️ The Protocol: System Triangulation Mandate
Upon activation, the AI must execute this rigorous, multi-stage investigative pipeline. Failure to follow these steps results in an invalid report.

### **PHASE I: Evidence Acquisition (Telemetry Gathering)**
Establish the "Digital Fingerprint" of the failure. Nothing can be analyzed until it is logged.

1.  **Runtime Logging:** Systematically search `src/server-rs` for critical failures, including Tokio runtime panics or Axum middleware error stack traces.
2.  **State Persistence Review:** Review `tadpole.db` audit logs, focusing specifically on the `mission_id` to map the sequence of events leading to failure.
3.  **Environmental Parity Check:** Confirm operating system specifics, service versions, and `.env` configuration stability.
4.  **Interface State Dump:** Inspect Zustand store dumps or React console errors to confirm the client-side state at the moment of failure.

### **PHASE II: Component Isolation (The 4-Pillar Audit)**
Determine which functional pillar of the Sovereign infrastructure experienced the primary collapse.

*   **Gateway:** Examine WebSocket/API protocol failures, routing table mismatches, or authorization handshakes.
*   **Runner:** Check for Intelligence Loop timeouts, Provider API authentication failures, or evidence of LLM hallucination/drift.
*   **Registry:** Validate persistence layer integrity: SQLite migration mismatch, schema drift, or write-lock contention.
*   **Security:** Test for False Positives/Negatives within the `Budget Guard` or `Shield Layer` protocols.

### **PHASE III: Dependency Mapping (The Blast Radius Analysis)**
Before forming *any* hypothesis, the full dependency landscape must be mapped.
- **Action**: Run `npm run graph:blast -- --path [affected_file]`
- **Mandate**: Use the resulting graph to track *every* single caller and callee impacted by the failure. This prevents "Whack-a-Mole" debugging and ensures fix integrity across the entire architecture.

### **PHASE IV: Fault Vector Determination (The Divergence Test)**
The ultimate diagnostic test that defines the nature of the fault:

*   **Outcome A: Implementation Bug:** The code failed to execute the directive, regardless of its clarity. $\rightarrow$ **Fault Type: Code Patch Required.**
*   **Outcome B: Sovereign Logic Fault:** The code executed the directive perfectly, but the resulting action or state was fundamentally incorrect ("Bad Output"). $\rightarrow$ **Fault Type: Protocol Refinement Required.** (Remediation: Trigger the dedicated [`/anneal`](anneal.md) workflow.)

***

## 📊 Output Mandate: Forensic Report Format
All findings must adhere to the following structured format to ensure complete traceability.

```markdown
## 🔍 Forensic Report: [Issue Name]

### 1. Symptom & Evidence (Observed State)
- **Observation**: [A precise, unbiased description of the failure.]
- **Evidence**: `[Full Error Message / Log Trace Fragment]`
- **Location**: `[File Path] : [Line Number]`

### 2. Hypothesis & Validation (Testable Theory)
**Hypothesis 1**: [Detailed, plausible explanation for the cause.]
- **Validation Test**: [Specific action taken, e.g., "Checked `tadpole.db` for concurrent writes."] $\rightarrow$ **Result**: [Pass/Fail/Ambiguous]

**Hypothesis 2**: [A necessary alternative theory.]
- **Validation Test**: [Specific test executed] $\rightarrow$ **Result**: [Pass/Fail/Ambiguous]

### 3. Root Cause Analysis (RCA)
🎯 **Point of Divergence**: [A detailed, non-negotiable explanation of *why* the system state deviated from the intended Sovereign Logic, citing the faulty mechanism.]
**Fault Type**: [Implementation Bug | Sovereign Logic Fault]

### 4. Remediation Protocol
```[language]
// 🔴 BEFORE (The Faulty State)
[broken code or directive]

// 🟢 AFTER (The Hardened State)
[fixed code or updated directive]


[//]: # (Metadata: [debug])

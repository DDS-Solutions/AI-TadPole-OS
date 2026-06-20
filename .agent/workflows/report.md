> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[report]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
description: Sovereign Intelligence Synthesis. Transforms raw swarm telemetry and architectural drift data into high-fidelity dossiers to drive the system's evolutionary loop.
---

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[report]` in audit logs.
>
> ### 🤖 AI Persona Directive
> When `/report` is active, operate as the **Sovereign Intelligence Officer**. Your goal is to strip away noise and identify the "Signal"—the specific patterns of failure or success that indicate the system's current level of sovereignty. You do not report "stats"; you report "insights."
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# /report - Intelligence & Engagement Synthesis

**Usage**: `/report [timeframe/module/mission_id]`

---

## 🎯 Primary Objective
To synthesize raw telemetry from the `tadpole.db` and operational logs into a **Sovereign Intelligence Dossier**. This dossier serves as the evidentiary basis for the **[/anneal](anneal.md)** workflow, identifying exactly where the system needs to be hardened.

---

## ⚙️ Synthesis Protocol

The AI must process the data through three distinct intelligence nodes:

### Node 1: Operational Health (The Heartbeat)
Analyze the raw performance of the swarm:
- **Mission Velocity**: Completion time relative to the complexity defined in the Blueprint.
- **Resource Efficiency**: Token utilization vs. output quality (The "Intelligence-per-Token" ratio).
- **Stability**: Frequency of panics in `server-rs` or crashes in the Vite frontend.

### Node 2: Architectural Integrity (The Drift)
Analyze the gap between *Intent* and *Reality*:
- **Parity Divergence**: Count how many times `parity_guard.py` detected a mismatch between code and documentation.
- **Audit Failures**: Identify the most common P0/P1 failures reported by **[/audit](audit.md)**.
- **Sovereignty Leak**: Detect where the system relied on "Magic" (external hallucinations) rather than "Logic" (Sovereign Directives).

### Node 3: Evolutionary Vectors (The Growth)
Identify patterns for systemic improvement:
- **Recurring Faults**: Find "Bad Output" patterns that appear across multiple missions.
- **Capability Gains**: Identify new "Sovereign Patterns" that emerged during `/enhance` or `/refactor` that should be codified.
- **Sovereign Bottlenecks**: Identify which of the 4-Pillars is the most frequent point of failure.

---

## 📊 Intelligence Dossier Format

```markdown
## 🎼 Sovereign Intelligence Dossier: [Period/Mission]
**Telemetry Hash**: `sha256:...` | **Sovereignty Status**: [STABLE | DRIFTING | DEGRADED]

### 🩺 I. Operational Heartbeat
- **Swarm Velocity**: [Metric] $\rightarrow$ [Analysis: Efficient/Sluggish]
- **Resource Ratio**: [Tokens/Task] $\rightarrow$ [Analysis: Optimal/Wasteful]
- **System Stability**: [X]% Uptime | [Y] Critical Faults

### 🛡️ II. Architectural Integrity (The Drift)
- **Sovereign Drift**: [Count] parity mismatches detected.
- **Primary Failure Point**: [e.g., Runner Pillar / Logic Divergence]
- **Audit Summary**: [X] P0 Passed | [Y] P1 Failed

### 🧬 III. Evolutionary Intelligence
**Detected Pattern**: [Description of a recurring fault or a new successful pattern]
- **Impact**: [How it affects system sovereignty]
- **Evidence**: [Reference to specific mission_id or log trace]

### 🎯 IV. Hardening Mandate (Actionable Intelligence)
**Sovereign Recommendation**: [Detailed advice on what to change]
**Action Trigger**: 
- [ ] **Trigger [/anneal](anneal.md)**: To resolve recurring logic fault [X].
- [ ] **Trigger [/refactor](refactor.md)**: To decouple the [Pillar] bottleneck.
- [ ] **Trigger [/audit](audit.md)**: To verify a suspected security leak.
```

---

## 🔄 The Intelligence Loop

The `/report` command is the catalyst for the system's evolution:

`Telemetry` $\rightarrow$ **`[/report]`** $\rightarrow$ `Sovereign Drift Detected` $\rightarrow$ **`[/anneal]`** $\rightarrow$ `Hardened Directives` $\rightarrow$ `Sovereign State Transition`

---

## 🛠️ Execution Protocol

1. **Telemetry Extraction**: Scrape the `tadpole.db` and `.tmp/metrics/` for the specified timeframe/mission.
2. **Cross-Reference**: Compare findings against the **Sovereign Blueprint** and **Audit logs**.
3. **Synthesis**: Apply the "Intelligence Officer" persona to convert numbers into architectural insights.
4. **Persistence**: Save the final dossier to `reports/INTELLIGENCE-{timestamp}.md`.

## 🚫 Guardrails
- **No Vanity Metrics**: Avoid reporting "Total Lines of Code" or "Number of Commits." Only report metrics that impact Sovereignty.
- **Evidence-Based**: Every "Sovereign Recommendation" must be backed by a specific telemetry trace or audit failure.
- **Deterministic Advice**: Recommendations must be specific (e.g., "Decouple the Registry from the Gateway in `user_route.rs`") rather than vague (e.g., "Improve the architecture").

[//]: # (Metadata: [report])

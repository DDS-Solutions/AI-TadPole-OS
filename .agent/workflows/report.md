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
description: Intelligence & Engagement Reporting. Synthesizes swarm metrics and performance data into high-fidelity dossiers.
---

# /report - Intelligence & Engagement Synthesis

$ARGUMENTS

---

## 🎯 Primary Objective
Synthesize raw mission data, token metrics, and architectural shifts into structured intelligence for human or node consumption.

## 🏗️ Reporting Nodes

### 1. Engagement Metrics
- **Mission Velocity**: Average completion time vs. complexity.
- **Success Rate**: Successful `/test` runs vs. regressions.

### 2. Sovereign Intelligence
- **Registry Growth**: New skills or directives added.
- **Drift Mitigation**: Number of parity corrections performed by `parity_guard.py`.

### 3. Financial/Resource Audit
- **Token Budget**: Current utilization across providers (Gemini/OpenAI/Ollama).
- **Latency Baseline**: Average hub response time.

---

## 🛠️ Execution Protocol

1. **Scrape Telemetry**: Extract metrics from `tadpole.db` and `.tmp/metrics/`.
2. **Synthesize Dossier**: Apply the `brainstorming` skill to highlight key intelligence vectors.
3. **Persist Findings**: Write to `reports/INTELLIGENCE-{timestamp}.md`.

---

## Output Format

```markdown
## 🎼 Intelligence Report: [Period]

### 🚀 Performance Highlights
- **Swarm Velocity**: [KM/h equivalent or task count]
- **Resource Efficiency**: [Optimal/Wasteful]

### 🧠 Swarm Evolution
- **New Capabilities**: [List]
- **Hardened Directives**: [List]

### 💡 Strategic Recommendation
[One paragraph of actionable architectural advice]
```

[//]: # (Metadata: [report])

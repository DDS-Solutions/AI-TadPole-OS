> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[scale_cluster]` in audit logs.
>
> ### AI Assist Note
> 🛰️ Directive: Scale Cluster (SOP-SYS-03)
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 🛰️ Directive: Scale Cluster (SOP-SYS-03)

## 🎯 Primary Objective
Govern the expansion and optimization of the Tadpole OS swarm to meet increasing computational or strategic demands. 

---

## 🏗️ Scaling Strategies

### 1. Vertical Scaling (Cognitive Depth)
- **Goal**: Increase the "IQ" of an active mission.
- **Action**: Upgrade mission sub-agents from lightweight local models (Ollama) to specialized high-fidelity models (Gemini-1.5-Pro, GPT-4o).
- **Metric**: Monitor `tokens_per_second` and "Reasoning Correctness" in `working_memory`.

### 2. Horizontal Scaling (Breadth)
- **Goal**: Parallel execution of non-dependent tasks.
- **Action**: Increase the `MAX_AGENTS` limit in `.env` and initialize additional "Worker" nodes.
- **Mechanism**: Use `FuturesUnordered` in `runner.rs` to execute more tool calls in parallel.

---

## 🛠️ Operational SOP

### 1. Resource Monitor
- **Tool**: `monitoring.rs` telemetry.
- **Check**: Audit RAM usage and CPU load on the host machine.
- **Guardrail**: If CPU > 90% or RAM > 85%, pause new mission spawns until existing tasks synthesize.

### 2. Cluster Sharding
- **Action**: Distribute missions across multiple `Bunker Nodes` using the `discovery.rs` registry.
- **Focus**: Ensure data parity across the distributed cluster.

---

## 📊 Evaluation Metrics
Track "Swarm Density" (Agents per Cluster) and "Mission Throughput" (Missions per Hour). Use these metrics to refine the `resource_allocation.md` policy for the next cycle.
[//]: # (Metadata: [scale_cluster])

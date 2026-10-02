---
title: "Continuity Scheduler"
tier: "1"
status: "verified"
version: "1.1.463"
last-verified: "2026-10-01"
commit: "bbcf0d4"
network-badge: "optional"
risk-tags:
  - "RISK: MEDIUM"
---

# ⏱️ Continuity Scheduler

⚡ OPTIONAL NETWORK

Continuity is the engine's job/workflow scheduler for recurring and on-demand swarm work. HTTP surface is nested under **`/v1/continuity/*`** (jobs CRUD, enable/disable, run-now, workflows, runs/cancel, workflow-run steps). The scheduler poll interval is **60s** (`server-rs/src/continuity/executor.rs`).

## Operator quick-ref

| Area | Notes |
|------|-------|
| OpenAPI | Continuity paths match the router on tip `bbcf0d4` / v1.1.463 |
| Module path | Workflow logic lives under `server-rs/src/continuity/workflow/` (directory module — not a single `workflow.rs` file) |
| Circuit breaker | Continuity breaker namespace is covered in `docs/OPERATIONS_MANUAL.md` |
| Deep dive | See [`docs/SWARM_ORCHESTRATION.md`](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/docs/SWARM_ORCHESTRATION.md) § Continuity / orchestration |

> [!NOTE]
> This wiki page is a pointer for operators. Prefer the main-repo Swarm Orchestration and OpenAPI docs for full request schemas.

→ *See [[AI-Tadpole-OS-Orchestration]] for Conductor / DAG context.*
→ *See [[Developer-Appendix]] for API index pointers.*

<!-- Last verified against commit bbcf0d4 on 2026-10-01 -->
[//]: # (wiki-page: Continuity)
[//]: # (Metadata: [wiki-ops])

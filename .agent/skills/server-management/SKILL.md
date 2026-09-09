---
name: server-management
description: Server management principles and decision-making. Process management, monitoring strategy, and scaling decisions. Teaches thinking, not commands.
when_to_use: "When managing servers, configuring process managers (PM2), setting up monitoring, or planning scaling strategies."
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / server-management
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Production Server Management Protocol

> **Purpose**: Process supervision, telemetry monitoring, log rotation, and infrastructure scaling.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** operations rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/pm2_and_systemd_configs.md`](./references/pm2_and_systemd_configs.md) | Full systemd unit files, PM2 clustering configurations, log rotation directives | Configuring daemon services & process restarts |

---

## ⚙️ 1. Process Supervision Strategy

| Application Layer | Supervisor Mechanism | Core Capabilities |
|---|---|---|
| **Rust Backend (`server-rs`)** | `systemd` / Docker | Native auto-restart, cgroup memory limits, file limits |
| **Node.js Gateway / Front** | `PM2` / `systemd` | Multi-core clustering, zero-downtime hot reload |

---

## 📊 2. Health Checks & Observability Baseline

- **Liveness Endpoint**: `/v1/health` must return `200 OK` and fast boolean status.
- **Readiness Check**: Inspect database connectivity (SQLite WAL mode active) before routing user traffic.
- **Log Sanitation**: Stream structured JSON logs (`tracing-subscriber`); mask secret tokens and PII.

---

## 📈 3. Scaling Decision Matrix

```
Performance Bottleneck:
├── CPU Saturation (90%+)  ➔ Horizontal scaling (Add cluster nodes)
├── Memory Growth / Leak   ➔ Profile allocations; set max_memory_restart
└── Latency Spikes         ➔ Inspect database indexes & connection pool
```
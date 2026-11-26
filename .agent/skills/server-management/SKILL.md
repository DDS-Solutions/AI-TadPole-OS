---
name: server-management
description: "Server management principles and decision-making. Process supervision, systemd units, PM2 clustering, health monitoring, and scaling strategies."
when_to_use: "When managing servers, configuring process managers (PM2/systemd), setting up health check endpoints, or planning scaling strategies."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: HIGH
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / server-management
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Zombie worker processes, unmonitored memory leaks, or missing process restart policies.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Production Server Management Protocol

> **Purpose**: Standardize process supervision, daemon auto-restarts, structured logging, health check endpoints, and infrastructure scaling across backend nodes.  
> **Workflow Bindings**: Integrated into [`/deploy`](../../workflows/deploy.md) and [`/status`](../../workflows/status.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign operations rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/pm2_and_systemd_configs.md`](./references/pm2_and_systemd_configs.md) | Full systemd unit files, PM2 clustering configurations, log rotation directives | Configuring daemon services & process restarts |
| [`scripts/health_monitor.py`](./scripts/health_monitor.py) | Automated health check validator and systemd unit configuration auditor | Validating health check payloads and supervisor configs |

---

## ⚙️ 1. Process Supervision Strategy

| Application Layer | Supervisor Mechanism | Core Capabilities |
|---|---|---|
| **Rust Backend (`server-rs`)** | `systemd` / Docker | Native auto-restart, cgroup memory limits, file limits (`LimitNOFILE=65536`) |
| **Node.js Gateway / Front** | `PM2` / `systemd` | Multi-core clustering, zero-downtime hot reload, memory threshold restarts |

---

## 📊 2. Health Checks & Observability Baseline

- **Liveness Endpoint**: `/v1/health` must return `200 OK` and fast boolean status (`{"status": "ok"}`).
- **Readiness Check**: Inspect database connectivity (SQLite WAL mode active) before routing incoming user traffic.
- **Log Sanitation**: Stream structured JSON logs (`tracing-subscriber`); mask secret tokens and customer PII.

---

## 🧠 3. Socratic Gate for Server Operations

Before deploying or configuring service processes, verify:
- **Gate 1 (Supervisor Bound)**: Is the service managed by a supervisor (`systemd`, PM2, Docker) with explicit `Restart=on-failure` policies?
- **Gate 2 (Resource Ceilings)**: Are memory limits (`max_memory_restart` or `MemoryMax`) and file descriptor limits defined?
- **Gate 3 (Health Telemetry)**: Does the process expose a low-overhead `/v1/health` endpoint that responds in $<10$ms?

---

## 🚫 4. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Unmanaged Processes** | Running `node server.js &` in a terminal crashes permanently on panic. | Manage via `systemd` unit or PM2 ecosystem file. |
| **Heavy Health Endpoints** | `/health` running complex DB table scans stalls under load. | Keep `/health` fast; separate deep diagnostics to `/health/readiness`. |
| **Uncapped Log Files** | Services logging to text files without logrotate exhaust disk space. | Configure `journald` or logrotate with `max-size 100M`. |
| **Ignoring File Descriptors** | High concurrency exhausts default 1024 open file limit. | Set `LimitNOFILE=65536` in systemd unit. |

---

## 🛠️ 5. Deterministic Verification Gate

Validate supervisor configurations and health payloads:

```powershell
# 1. Run built-in self-tests
python .agent/skills/server-management/scripts/health_monitor.py --test

# 2. Audit a systemd service unit file
python .agent/skills/server-management/scripts/health_monitor.py --check-unit /etc/systemd/system/tadpole.service
```
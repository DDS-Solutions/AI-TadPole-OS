---
title: "Kill Switches"
tier: "3"
status: "reviewed"
version: "1.1.464"
last-verified: "2026-11-20"
network-badge: "none"
risk-tags:
  - "RISK: HIGH"
  - "RISK: DESTRUCTIVE"
commit: "3eef5c2d"
---

# 🛑 Kill Switches

> [!CAUTION]
> This page documents **emergency intervention controls** that immediately halt agent execution or terminate the engine process. Use these controls only when standard oversight mechanisms are insufficient.

These controls are reachable in **≤ 2 clicks from the Sidebar** (Tier 3 → Kill-Switches).

---

## 1. Emergency Controls

### 1.1 Halt Agents (`kill_agents_admin` & `resume_engine`)

Immediately halts all active agent thinking and execution loops, rejects pending oversight requests, and durably latches `dispatch_paused: true` in `data/governance_settings.json` until an administrator calls `POST /v1/engine/resume`.

| Property | Value |
|----------|-------|
| **Backend Handler** | `kill_agents_admin` → `kill_agents` ([engine_control.rs:L147](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/server-rs/src/routes/engine_control.rs#L147)) |
| **Resume Handler** | `resume_engine` ([engine_control.rs:L155](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/server-rs/src/routes/engine_control.rs#L155)) |
| **Frontend Hook** | `handle_kill_switch` ([useOversightDashboard.ts:L262](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/src/hooks/useOversightDashboard.ts#L262)) / `handle_resume_engine` (`L286`) |
| **API Endpoints** | `POST /v1/engine/kill` (`RequireAdmin`), `POST /v1/engine/resume` (`RequireAdmin`), paired mobile `POST /v1/remote/agents/halt` |
| **Dashboard Button Path** | Oversight dashboard → header panel → **[Button] Halt Agents** / **Resume Engine** |

**[Button] Halt Agents**
- *Default state*: Enabled/Online.
- *Visible location*: Oversight dashboard header panel.
- *Observable side effect*: Prompts the user with `confirm_halt_agents` dialog, then halts all active agent thinking loops and latches `dispatch_paused: true` via `handle_kill_switch` (requires `NEURAL_ADMIN_TOKEN` / browser Step-Up Administrative Token).
- *Keyboard shortcut*: None.
- *Missing in build*: Always present.

**CLI Commands (Admin Token Required — `$NEURAL_TOKEN` returns `403 Forbidden`):**
```powershell
# Halt all agents and latch dispatch_paused
curl -X POST http://localhost:8000/v1/engine/kill -H "Authorization: Bearer $NEURAL_ADMIN_TOKEN"

# Explicitly release dispatch_paused latch and resume task dispatch
curl -X POST http://localhost:8000/v1/engine/resume -H "Authorization: Bearer $NEURAL_ADMIN_TOKEN"
```

> [!CAUTION]
> This immediately aborts all running agent tasks and blocks new dispatches across restarts until `POST /v1/engine/resume` is invoked by an administrator. Use the Oversight Queue's individual **Reject** buttons for targeted intervention first.

---

### 1.2 Kill Engine (`shutdown_engine`)

Gracefully terminates the Axum service process, persisting all live agent states to the database before exit (`RequireAdmin`).

| Property | Value |
|----------|-------|
| **Backend Handler** | `shutdown_engine` ([engine_control.rs:L190](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/server-rs/src/routes/engine_control.rs#L190)) |
| **Frontend Hook** | `handle_kill_engine` ([useOversightDashboard.ts:L309](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/src/hooks/useOversightDashboard.ts#L309)) |
| **API Endpoint** | `POST /v1/engine/shutdown` (`RequireAdmin`) |
| **Dashboard Button Path** | Oversight dashboard → header panel → **[Button] Kill Engine** |

**[Button] Kill Engine**
- *Default state*: Enabled/Online.
- *Visible location*: Oversight dashboard header panel.
- *Observable side effect*: Prompts the user with `confirm_kill_engine` verification, **demands typing the uppercase word `"SHUTDOWN"` in the text input** ([useOversightDashboard.ts:L312-313](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/src/hooks/useOversightDashboard.ts#L312-L313)), and then terminates the Axum service process via `shutdown_engine`.
- *Keyboard shortcut*: None.
- *Missing in build*: Always present.

**CLI Command (Admin Token Required):**
```powershell
curl -X POST http://localhost:8000/v1/engine/shutdown -H "Authorization: Bearer $NEURAL_ADMIN_TOKEN"
```

> [!CAUTION]
> This terminates the entire backend process. The dashboard will lose connection and show **🔴 OFFLINE**. Restart the engine manually with `cargo run --release`.

---

## 2. Decision Matrix: Which Control to Use

| Scenario | Recommended Control | Reason |
|----------|-------------------|--------|
| Single agent misbehaving | **[Button] Reject** in Oversight Queue | Targeted, non-destructive |
| Multiple agents out of control | **[Button] Halt Agents** (`handle_kill_switch`) | Stops all agents, preserves engine |
| Engine unresponsive or security breach | **[Button] Kill Engine** (`handle_kill_engine`) | Full process termination |
| Budget reservation denied | The affected provider call is not dispatched; the runner reports `BUDGET_EXHAUSTED` (HTTP 402) | Per-agent/per-mission quota enforcement; does not globally halt all agents |

---

## 3. Circuit Breaker (Client-Side)

In addition to server-side kill switches, the frontend implements a **[[Circuit Breaker|Glossary#circuit-breaker]]** system that automatically protects the dashboard:

### 3.1 Breaker States

| State | Behavior |
|-------|----------|
| **CLOSED** | Normal operation. All API requests pass through. |
| **OPEN** | Triggered after `VITE_SYSTEM_BREAKER_FAILURE_THRESHOLD` [RISK: HIGH] (default: `5`) consecutive failures. Further calls are instantly rejected with `CircuitBreakerOpenError`. |
| **HALF_OPEN** | After `VITE_SYSTEM_BREAKER_COOLDOWN_MS` [RISK: HIGH] (default: `10000` ms) cooldown, a single probe request tests backend health. Two successes reset to CLOSED; one failure trips back to OPEN. |

### 3.2 Log Observability

- 📡 `[Circuit Breaker] NAMESPACE entered HALF_OPEN probe state` — Cooldown expired; probing service.
- ❌ `[Circuit Breaker] NAMESPACE tripped back to OPEN from HALF_OPEN due to trial failure` — Service still down.

---

## 4. Local-First Pledge

> [!IMPORTANT]
> Halt and shutdown controls affect the local engine process and dispatch state. They are not network-egress controls and do not cancel requests already sent to external providers. The engine may continue recording local telemetry or sending configured service traffic while it remains running.

---

→ *See [[Approvals-&-Quotas]] for standard oversight controls.*
→ *See [[Network-Boundaries]] and [[Network-Boundaries]] for CORS and proxy configuration.*
→ *See [[Configuration]] for network and breaker environment variables.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/docs/GLOSSARY.md). Every `[[term|Glossary#term]]` link on this page resolves to an entry there.

<!-- Last verified against commit 3eef5c2d on 2026-11-02 -->
[//]: # (wiki-page: Kill-Switches)


[//]: # (Metadata: [wiki-security])

---
title: "Kill Switches"
tier: "3"
status: "verified"
version: "1.2.0"
last-verified: "2026-06-17"
commit: "b1c347b1"
network-badge: "none"
risk-tags:
  - "RISK: HIGH"
  - "RISK: DESTRUCTIVE"
---

# 🛑 Kill Switches

> [!CAUTION]
> This page documents **emergency intervention controls** that immediately halt agent execution or terminate the engine process. Use these controls only when standard oversight mechanisms are insufficient.

These controls are reachable in **≤ 2 clicks from the Sidebar** (Tier 3 → Kill-Switches).

---

## 1. Emergency Controls

### 1.1 Halt Agents (`kill_agents`)

Immediately halts all active agent thinking and execution loops and rejects pending oversight requests.

| Property | Value |
|----------|-------|
| **Backend Handler** | `kill_agents` ([engine_control.rs:L34](../../../server-rs/src/routes/engine_control.rs#L34)) |
| **Frontend Hook** | `handle_kill_switch` ([useOversightDashboard.ts:L153](../../../src/hooks/useOversightDashboard.ts#L153)) |
| **API Endpoint** | `POST /v1/engine/kill` |
| **Dashboard Button Path** | Oversight dashboard → header panel → **[Button] Halt Agents** |

**[Button] Halt Agents**
- *Default state*: Enabled/Online.
- *Visible location*: Oversight dashboard header panel.
- *Observable side effect*: Prompts the user with `confirm_halt_agents` dialog, then halts all active agent thinking loops by sending thread abort signals via the `handle_kill_switch` handler.
- *Keyboard shortcut*: None.
- *Missing in build*: Always present.

**CLI Command:**
```powershell
curl -X POST http://localhost:8000/v1/engine/kill -H "Authorization: Bearer $NEURAL_TOKEN"
```

> [!CAUTION]
> This immediately aborts all running agent tasks. In-progress work may be lost. Use the Oversight Queue's individual **Reject** buttons for targeted intervention first.

---

### 1.2 Kill Engine (`shutdown_engine`)

Gracefully terminates the Axum service process, persisting all live agent states to the database before exit.

| Property | Value |
|----------|-------|
| **Backend Handler** | `shutdown_engine` ([engine_control.rs:L91](../../../server-rs/src/routes/engine_control.rs#L91)) |
| **Frontend Hook** | `handle_kill_engine` ([useOversightDashboard.ts:L169](../../../src/hooks/useOversightDashboard.ts#L169)) |
| **API Endpoint** | `POST /v1/engine/shutdown` |
| **Dashboard Button Path** | Oversight dashboard → header panel → **[Button] Kill Engine** |

**[Button] Kill Engine**
- *Default state*: Enabled/Online.
- *Visible location*: Oversight dashboard header panel.
- *Observable side effect*: Prompts the user with `confirm_kill_engine` verification, **demands typing the uppercase word `"SHUTDOWN"` in the text input** ([useOversightDashboard.ts:L172-173](../../../src/hooks/useOversightDashboard.ts#L172-L173)), and then terminates the Axum service process via the `shutdown_engine` handler.
- *Keyboard shortcut*: None.
- *Missing in build*: Always present.

**CLI Command:**
```powershell
curl -X POST http://localhost:8000/v1/engine/shutdown -H "Authorization: Bearer $NEURAL_TOKEN"
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
| Budget overspend detected | Budget Guard auto-halts | Automatic via `BudgetExhausted` error |

---

## 3. Circuit Breaker (Client-Side)

In addition to server-side kill switches, the frontend implements a **[[Glossary#circuit-breaker|Circuit Breaker]]** system that automatically protects the dashboard:

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
> **100% Local-First / Zero Data Leaks.**
> - All emergency controls are executed locally. No telemetry or execution signals are transmitted outside the local network.
> - The shutdown controls and state persistence checks are processed entirely on your local machine.

---

→ *See [[Approvals-&-Quotas|§1]] for standard oversight controls.*
→ *See [[Network-Boundaries|§2]] and [[Network-Boundaries|§3]] for CORS and proxy configuration.*
→ *See [[Configuration|§4]] for network and breaker environment variables.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](../GLOSSARY.md). Every `[[Glossary#term|term]]` link on this page resolves to an entry there.

<!-- Last verified against commit b1c347b1 on 2026-06-11 -->
[//]: # (wiki-page: Kill-Switches)


[//]: # (Metadata: [wiki-security])

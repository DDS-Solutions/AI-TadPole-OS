> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Core Docs / WEBSOCKET_EVENTS
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py`

# 📡 WebSocket Communication: Real-time Telemetry

> **@docs ARCHITECTURE:Observability**
> **Last Verified**: 2026-10-01 vs `server-rs/src/routes/ws.rs` + `router.rs`

Tadpole OS exposes **two** WebSocket entrypoints. The telemetry hub is bidirectional for a **small** set of app-level commands; agent chat and user-answer flows use **REST**, not WS.

---

## 🔌 Connection Hubs

| Route | Purpose |
| :--- | :--- |
| `ws://localhost:8000/v1/engine/ws` | Telemetry / logs / swarm events / oversight decisions |
| `ws://localhost:8000/v1/engine/live-voice` | Gemini Live multimodal proxy (audio/setup); see Live Voice notes |

**Auth (both):** `Sec-WebSocket-Protocol: bearer.<NEURAL_TOKEN>` (same bearer family as HTTP). Failed auth → `auth_error` then close; success → `auth_ok`.

### Multiplexing Strategy (`/v1/engine/ws`)

The engine uses `tokio::select!` to multiplex internal broadcast channels over a single socket:

1. **System Logs** — server and agent mission logs
2. **Engine Events** — lifecycle events (`agent:create`, `oversight:new`, …)
3. **Telemetry Pulse** — high-speed binary updates for visualization

Server also emits periodic **JSON `heartbeat`** frames and protocol-level WebSocket **Ping**; clients may send app-level `{"type":"ping"}` and receive `{"type":"pong",...}`.

---

## 📡 Subscribing to Events (Server → Client)

### 1. Session / liveness frames

| Event Type | Description |
| :--- | :--- |
| `auth_ok` | First successful auth acknowledgment |
| `auth_error` | Invalid credentials (`message` field) |
| `heartbeat` | Periodic liveness JSON from the send loop |
| `pong` | Echo response to client `ping` |

### 2. JSON Event Bus

| Event Type | Description |
| :--- | :--- |
| `agent:status` | "Thinking", "Invoking Tool", or "Idle" transitions |
| `agent:message` | Incremental text chunks from the LLM |
| `agent:reasoning_step` | Structured reasoning traces (model slot, lineage, access lists) |
| `agent:execution_metrics` | Live per-mission metrics (turns, tools, tokens, cost) |
| `agent:user_question` | Structured question with options for the operator (**answer via REST**, not WS) |
| `oversight:new` | Pending tool-execution approval |
| `trace:span` | OTel-compliant span initiation |

### 3. Reasoning Step Telemetry

```json
{
  "type": "agent:reasoning_step",
  "agent_id": 42,
  "mission_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "step": {
    "id": "step-1",
    "parent_id": null,
    "content": "Analyzing codebase parameters...",
    "model": "gpt-4-turbo",
    "slot": "Primary",
    "lineage": [42],
    "access_list": [1, 2]
  }
}
```

- **`lineage`**: agent IDs on the recruitment path
- **`access_list`**: Conductor DAG step IDs this agent may read
- **`slot`**: `Primary` / `Secondary` / `Tertiary`

### 4. Binary "Swarm Pulse" (MessagePack)

- **Frequency**: ~10Hz (default telemetry cadence)
- **Header**: prefixed with `0x02` (binary pulse marker) in `ws.rs`
- **Payload**: MessagePack-encoded agent metrics

---

## 📡 Sending Commands (Client → Server)

`ws.rs` handles **only** these inbound JSON `type` values on `/v1/engine/ws`:

| `type` | Status |
| :--- | :--- |
| `ping` | Implemented — triggers `pong` |
| `oversight:decision` | Implemented — flat fields (see below) |
| `agent:send` | **Not implemented on WS** — use REST (`POST /v1/agents/{id}/tasks` or chat UI) |
| `agent:user_answer` | **Not implemented on WS** — submit via REST / oversight APIs |

### 1. App-level ping

```json
{ "type": "ping" }
```

### 2. Submit signed oversight decision (actual wire shape)

Fields are **flat** on the message object (not nested under `payload`). Decisions are lowercase `approved` / `rejected`.

```json
{
  "type": "oversight:decision",
  "id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "decision": "approved",
  "signature": "8a3a2f...",
  "verifying_key": "f3b20c...",
  "timestamp": 1716839064,
  "nonce": "optional-nonce",
  "override_slot": null,
  "user_answer": null
}
```

Ed25519 signature validation applies; invalid / shed decisions are logged and ignored.

---

## 🎙️ Live Voice (`/v1/engine/live-voice`)

Specialized upgrade that proxies client audio/setup to Google's Gemini Live backend (API keys stay server-side). Same bearer subprotocol auth as the telemetry hub.

> [!WARNING]
> **UI auth caveat:** `Live_Voice_Hub.tsx` reads `localStorage.getItem('tadpole_token')` and falls back to `bearer.anonymous`. Settings stores `tadpole_os_api_key` **memory-only** (not that localStorage key). Unless something else writes `tadpole_token`, Live Voice may authenticate as anonymous / fail depending on server policy. Prefer setting `tadpole_token` to match `NEURAL_TOKEN` for experiments, or use a client that passes the Settings bearer — code fix tracked separately from docs.

---

## ⚡ Performance: RAF Batching

During complex swarms the hub can emit hundreds of events/sec. Client UIs should **requestAnimationFrame**-batch socket events and flush once per frame.

## 🔄 Stream-then-Poll Fallback

- **Primary**: `/v1/engine/ws`
- **Degradation**: on heartbeat miss / unexpected close during a mission, poll `GET /v1/agents/{id}/mission` (adaptive 1–3s) with a monotonic cursor
- **Resume**: WebSocket reconnect continues; dedupe against cursor history

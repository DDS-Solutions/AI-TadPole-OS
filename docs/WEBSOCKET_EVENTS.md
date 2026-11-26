> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Core Docs / WEBSOCKET_EVENTS
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py`

# 📡 WebSocket Communication: Real-time Telemetry

> **@docs ARCHITECTURE:Observability**

The Tadpole OS Engine utilizes a single, high-concurrency WebSocket hub for real-time telemetry, system logs, and swarm events.

---

## 🔌 Connection Hub

- **Telemetry Endpoint**: `ws://localhost:8000/v1/engine/ws` (Rust Hub)
- **Live Voice Endpoint**: `ws://localhost:8000/v1/engine/live-voice` (Gemini Live proxy — strictly requires `Sec-WebSocket-Protocol: bearer.<token>` pre-auth and returns `403 Forbidden` when Privacy Shield is active)
- **Telemetry Auth**: Supports both `Sec-WebSocket-Protocol: bearer.<token>` during upgrade AND post-connect JSON `{"type":"auth","token":"<NEURAL_TOKEN_OR_NEURAL_ADMIN_TOKEN>"}` within `WS_AUTH_TIMEOUT` (5 seconds, responding with `auth_ok` or `auth_error`). Post-connect auth is restricted exclusively to `/v1/engine/ws`.
- **Close Codes**: `4401` (unauthorized), `4402` (invalid auth frame), `4408` (auth timeout), `4409` (idle timeout — 120s without heartbeat), `4410` (server shutdown).

### Multiplexing Strategy
The engine uses `tokio::select!` (`routes/ws/telemetry/send_task.rs`) to multiplex five internal broadcast channels over a single socket (with 30s heartbeats and 120s idle timeout):
1.  **System Logs (`log_rx`)**: Real-time server and agent mission logs.
2.  **Engine Events (`event_rx`)**: High-level lifecycle events (e.g., `agent:create`, `oversight:new`, `mission_update`).
3.  **Telemetry Spans (`telemetry_rx`)**: Distributed tracing spans and execution metrics.
4.  **Audio Stream (`audio_rx`)**: Binary audio frames prefixed with `0x01`.
5.  **Swarm Pulse (`pulse_rx`)**: High-speed 10Hz MessagePack telemetry prefixed with `0x02`.

---

## 📡 Subscribing to Events (Server → Client)

### 1. JSON Event Bus
Most interaction events are broadcast as standard JSON objects.

| Event Type | Description |
|:--- |:--- |
| `agent:status` | Broadcasts "Thinking", "Invoking Tool", or "Idle" transitions. |
| `agent:message` | Streams incremental text chunks from the LLM. |
| `agent:reasoning_step` | Broadcasts structured reasoning traces, indicating model slot, execution lineage, and task step access lists. |
| `agent:execution_metrics` | Broadcasts live per-mission execution metrics (turns, tool calls, cache hits, token usage, cost, summarization warnings). |
| `agent:user_question` | Dispatches structured question with options from an agent to the operator. |
| `oversight:new` | Notifies the dashboard of a pending tool execution approval. |
| `mission_update` | Broadcasts mission lifecycle status changes (`active`, `completed`, `paused`) and pin updates. |
| `trace:span` | OTel-compliant span initiation for distributed tracing. |

### 2. Reasoning Step Telemetry
During reasoning loops, the engine emits `agent:reasoning_step` events to track details of the model execution:

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

Key Fields:
- **`lineage`**: An array of agent IDs tracing the recursive path of recruitment (e.g. `[1, 2, 42]`), preventing loop recruitment.
- **`access_list`**: An array of step IDs from the Conductor DAG plan that this agent is authorized to read findings from, enforcing strict context boundary routing.
- **`slot`**: The active model slot (`Primary`, `Secondary`, `Tertiary`) currently executing, tracking slot swapping.

### 3. The Binary "Swarm Pulse" (MessagePack)
To maintain 60FPS in the **Swarm Visualizer** without saturating the main thread, the engine broadcasts high-speed telemetry in binary format.
- **Frequency**: 10Hz (Default).
- **Header**: Prefixed with `0x02` (Binary Pulse marker).
- **Payload**: MessagePack-encoded agent metrics (CPU, Memory pressure, Battery, Active relationships).

---

## 📡 Sending Commands (Client → Server)

The telemetry socket (`routes/ws/telemetry/recv_task.rs`) accepts JSON objects with a `type` field (`"ping"`, `"auth"`, or `"oversight:decision"`). Agent messages and task dispatch use REST (`POST /v1/agents/{id}/tasks`).

### 1. Post-Connect Authentication (`/v1/engine/ws` only)
```json
{
  "type": "auth",
  "token": "<NEURAL_TOKEN_OR_NEURAL_ADMIN_TOKEN>"
}
```

### 2. Heartbeat Ping
```json
{
  "type": "ping"
}
```

### 3. Submit Signed Oversight Decision / Agent Question Answer (**Admin Role Required**)
Operator approval/rejection decisions and `ask_user_question` answers can be submitted over `/v1/engine/ws` **only when authenticated with `AuthenticatedRole::Admin` (`NEURAL_ADMIN_TOKEN`)**. In production, Ed25519 signature verification (`ovs:v2`) pinned to `OVERSIGHT_PUBLIC_KEY` with a single-use `nonce` is enforced:
```json
{
  "type": "oversight:decision",
  "id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "decision": "approved",
  "timestamp": 1716839064,
  "nonce": "f4c1a2e8b9d04721",
  "signature": "8a3a2f...",
  "verifying_key": "f3b20c...",
  "override_slot": null,
  "user_answer": "Option B: Use PostgreSQL instead of SQLite"
}
```
> `decision` must be strictly `"approved"` or `"rejected"` (top-level fields — no nested `"payload"` wrapper). Non-admin sockets receive an authorization error and the decision is rejected.

---

## ⚡ Performance Optimization: RAF Batching
The engine can emit hundreds of events per second during complex swarms.
- **Recommendation**: Client-side implementations should use a **requestAnimationFrame (RAF)** batching strategy to buffer incoming socket events and flush them to the UI only once per frame to prevent viewport jank.

---

## 🔄 Resilient Stream-then-Poll Fallback
To ensure mission observability is never lost when network conditions degrade:
- **Primary Transport**: Real-time bidirectional streaming over `/v1/engine/ws`.
- **Degradation Detection**: If socket heartbeat misses or connection terminates unexpectedly during an in-flight mission, the client enters fallback mode.
- **REST Polling**: Automatically syncs mission state via `POST /v1/agents/{id}/mission` at an adaptive interval (1s–3s) using a monotonic sequence cursor to prevent duplicate chunks.
- **Seamless Resumption**: Upon successful WebSocket reconnect, the stream seamlessly takes over, deduplicating incoming events against the cursor history.
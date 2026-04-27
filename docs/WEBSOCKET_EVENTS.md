> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Documentation**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Automated governance and architectural tracking.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 📡 WebSocket Communication: Real-time Telemetry

> **@docs ARCHITECTURE:Observability**

The Tadpole OS Engine utilizes a single, high-concurrency WebSocket hub for real-time telemetry, system logs, and swarm events.

---

## 🔌 Connection Hub

- **Endpoint**: `ws://localhost:8000/v1/engine/ws` (Rust Hub)
- **Auth**: Requires `bearer.<NEURAL_TOKEN>` in the `Sec-WebSocket-Protocol` header.

### Multiplexing Strategy
The engine Uses `tokio::select!` to multiplex three distinct internal broadcast channels over a single socket:
1.  **System Logs**: Real-time server and agent mission logs.
2.  **Engine Events**: High-level lifecycle events (e.g., `agent:create`, `oversight:new`).
3.  **Telemetry Pulse**: High-speed binary updates for visualization.

---

## 📡 Subscribing to Events (Server → Client)

### 1. JSON Event Bus
Most interaction events are broadcast as standard JSON objects.

| Event Type | Description |
|:--- |:--- |
| `agent:status` | Broadcasts "Thinking", "Invoking Tool", or "Idle" transitions. |
| `agent:message` | Streams incremental text chunks from the LLM. |
| `oversight:new` | Notifies the dashboard of a pending tool execution approval. |
| `trace:span` | OTel-compliant span initiation for distributed tracing. |

### 2. The binary "Swarm Pulse" (MessagePack)
To maintain 60FPS in the **Swarm Visualizer** without saturating the main thread, the engine broadcasts high-speed telemetry in binary format.
- **Frequency**: 10Hz (Default).
- **Header**: Prefixed with `0x02` (Binary Pulse marker).
- **Payload**: MessagePack-encoded agent metrics (CPU, Memory pressure, Battery, Active relationships).

---

## 📡 Sending Commands (Client → Server)

Commands should be sent as JSON objects with a `type` field.

```json
{
  "type": "agent:send",
  "agent_id": "agent-42",
  "message": "Continue the mission."
}
```

---

## ⚡ Performance Optimization: RAF Batching
The engine can emit hundreds of events per second during complex swarms.
- **Recommendation**: Client-side implementations should use a **requestAnimationFrame (RAF)** batching strategy to buffer incoming socket events and flush them to the UI only once per frame to prevent viewport jank.

[//]: # (Metadata: [WEBSOCKET_EVENTS])

[//]: # (Metadata: [WEBSOCKET_EVENTS])

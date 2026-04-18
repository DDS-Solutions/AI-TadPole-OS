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

# ⚛️ Frontend & Client Architecture

> **@docs ARCHITECTURE:Interface**

The Tadpole OS Dashboard is a high-concurrency React 19 interface designed for low-latency observability and multi-monitor swarm management.

---

## 🏗️ Reactive Infrastructure: Zustand

Tadpole OS utilizes a decentralized **Zustand** store architecture to maintain a unitary source of truth across all components.

| Store | Responsibility |
|:--- |:--- |
| **`agent_store`** | Identity, telemetry, and capability status. |
| **`workspace_store`** | Mission clustering and RAG context management. |
| **`provider_store`** | Secure credential management (Vault) and model availability. |
| **`tab_store`** | Viewport orchestration and layout state. |

### Performance Optimization: Atomic Selectors
To prevent re-render fatigue, components utilize **Atomic Selectors**. Updating a single agent's token-burn count will not trigger a re-render of the entire agent grid.

---

## 🖇️ The "State-Preserved Detachment" Pattern

Tadpole OS implements a high-performance multi-window system using **React Portals**.

### 1. Unified React Tree
Detached windows (e.g., Swarm Visualizer, Terminal) are NOT independent application instances. They exist within the **same React tree** and share the same JavaScript heap.

### 2. Zero-Latency Sync
Because common state (Zustand) is shared directly in memory, telemetry updates at 10Hz are reflected across all monitor screens with zero IPC (Inter-Process Communication) overhead.

### 3. Dynamic Style Synchronization
A `MutationObserver` in the parent window monitors Tailwind and theme changes, automatically injecting new styles into the detached windows in real-time.

---

## ⚡ Performance Buffering: RAF Batching

During high-concurrency agent swarms, the engine may broadcast hundreds of log events per second.
- **Event Bus**: All incoming WebSocket telemetry is pushed into a non-reactive `ref` bus.
- **RAF Flush**: A `requestAnimationFrame` hook flushes these events to the React state exactly once per frames (60fps), ensuring a smooth, jank-free "living" dashboard.

---

## 🛠️ Connectivity: WebSocket Multiplexing

The dashboard maintains a single persistent `/ws` connection with the engine.
- **Binary Protocols**: High-speed telemetry (e.g., the Swarm Pulse) is broadcast via **MessagePack** binary headers for minimal bandwidth overhead.
- **Auth Handshake**: Authentication is handled via `Sec-WebSocket-Protocol` headers, preventing token exposure in URL logs.

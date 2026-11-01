> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Core Docs / Frontend_State_Management
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py`

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
| **`provider_store`** | Provider identities and configuration metadata; encrypted credentials belong to `vault_store`. |
| **`model_store`** | Model inventory and selectors; mutations delegate to `services/model_commands.ts`. |
| **`tab_store`** | Viewport orchestration and layout state. |
| **`sovereign_store`** | Chat transcript facade, question actions, and persisted chat preferences. |

### Command and transition ownership (2026-10-22)

- **Provider commands**: `domain/providers/configuration.ts` validates drafts and models credential intent as keep, set, or remove. `services/provider_configuration.ts` serves Save, Test, and Discover. Blank key input keeps the saved credential; removal is explicit. Discovery requires endpoint, protocol, and headers to be saved first. Overlapping saves for one provider are rejected. A failed save restores affected metadata and ciphertext without replacing unrelated providers or vault entries.
- **Model commands**: `services/model_commands.ts` owns inventory writes and selective optimistic recovery. Synchronization includes pricing and quota changes; explicit local routing is retained.
- **Agent ingestion**: `domain/agents/state_reducer.ts` handles API snapshots and socket patches. Snapshots preserve client ordering metadata; patches preserve omitted fields and apply explicit nulls. Socket timestamps reject older updates, but do not establish total ordering across transports or ownership of every optimistic field.
- **Transcript**: `domain/chat/transcript.ts` owns message IDs, indexes, replay deduplication, and pruning. At 5,000 messages it removes the oldest 200 before adding the next message. Repeated question parts retain the existing answer.
- **Question commands**: `services/chat/question_decisions.ts` submits the exact question and choice. Failure removes its optimistic operator message and resets only the matching answer.
- **Replication and logs**: `services/chat/replication.ts` owns reference-counted channel subscriptions and cleanup. `services/chat/log_projection.ts` projects diagnostic events into chat. Event-bus deliveries distinguish local, remote, and replay origins so incoming or replayed logs are not rebroadcast.

The public Zustand stores remain the UI entry points. These boundaries do not introduce a new server event protocol. Detached React portals share the same store; independent application windows use the replication channel.

### Architectural Service Decoupling & Defensive State Management
- **Safe Store Injection & Defensive Access**: `workspace_service.ts` uses private initialization guards (`get_store()`) eliminating unhandled null dereferences (`workspace_store!`).
- **High-Entropy Cluster ID Generation**: Switched from truncated UUIDs to full 128-bit `crypto.randomUUID()` identifiers (`cl-${crypto.randomUUID()}`), eliminating collision vectors.
- **Optimistic State Rollback**: `update_budget` and `update_cluster_privacy` save previous state snapshots and automatically revert local store state upon backend API failure.
- **API Leakage Elimination**: Store actions delegate governance policy updates to `workspace_service.ts` and `system_api_service`, maintaining strict layer separation.
- **Agent deletion**: Store and dashboard actions use the same persistence service. Agents and workspace assignments are removed only after backend confirmation; failed deletions preserve local state and expose an error.
- **Registry refresh**: Backend invalidation events clear the cached registry before loading fresh data. Shared registry reads isolate each caller's cancellation, and failures from superseded requests cannot clear a newer cache entry.

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
- **Reconnect limits**: Failed handshakes retain their retry count and increasing delay. Successful authentication resets the retry budget.

---

## 🎨 Generative & Interactive UI Components

Tadpole OS provides typed, declarative generative UI and in-chat interactive widgets:

### 1. OpenUI Generative DSL Renderer (`OpenUI_Renderer.tsx`)
- **Safe Data-Only Rendering**: Renders dynamic layouts (`kpi_card`, `bar_chart`, `table`, `layout`) from structured JSON DSL payloads without `eval` or dangerously-set HTML.
- **Table Sorting & Pagination**: In-memory column sorting and pagination directly inside the chat stream.

### 2. Socratic Question Pills (`Question_Choice_Pill.tsx`)
- **Interactive Choice Dispatch**: Renders interactive multiple-choice buttons when the agent halts at a Socratic gate or asks a user clarification question.
- **Direct Store Binding**: Dispatches selections directly back to the active agent runner via WebSocket or REST bridge.

### 3. Mission Metrics HUD (`Mission_Metrics_Badge.tsx`)
- **Real-Time Execution Telemetry**: Displays token consumption, estimated USD expenditure, cache hit ratios, and turn counts.
- **Operational Health Warnings**: Visual indicator alerts when agent triggers $\ge 3$ context summarizations, signaling context bloat.

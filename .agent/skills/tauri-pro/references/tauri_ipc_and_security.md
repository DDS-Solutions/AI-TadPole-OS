# Tauri IPC, Security & Window Lifecycle Architecture (L3 Reference)

> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / tauri-pro / References
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Unhandled panics in IPC commands, insecure command exposure, window state loss.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

This document details the production IPC contract, event system, and security bounds for Tadpole OS's Tauri shell.

---

## 1. Rust `#[tauri::command]` Invariants

Every command exposed to the web frontend must follow strict error and type conventions:
1. **Never Return Raw Panics**: Always return `Result<T, AppError>` or `Result<T, String>`. Never call `.unwrap()` or `.expect()` inside a command handler.
2. **State Injection via `tauri::State`**: Inject shared managed state using `tauri::State<'_, Arc<AppState>>`.
3. **Async Offloading**: For CPU-heavy or blocking I/O work, mark commands as `async fn` and use `tokio::task::spawn_blocking`.

```rust
#[tauri::command]
pub async fn execute_mission(
    mission_id: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<MissionResult, String> {
    state.mission_manager.dispatch(&mission_id).await.map_err(|e| e.to_string())
}
```

---

## 2. Frontend IPC Invocation & Event Listeners

In the React SPA frontend, invoke commands via typed wrappers:

```typescript
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// 1. Typed Command Invocation
export async function runMission(missionId: string): Promise<MissionResult> {
  return await invoke<MissionResult>('execute_mission', { missionId });
}

// 2. Event Subscription with Clean Teardown
useEffect(() => {
  const unlistenPromise = listen<TelemetryPayload>('telemetry:stream', (event) => {
    updateMetrics(event.payload);
  });
  return () => {
    unlistenPromise.then((unlisten) => unlisten());
  };
}, []);
```

---

## 3. Capability Permissions & Scopes

- Declare permissions in `src-tauri/capabilities/default.json`.
- Restrict filesystem access to `$APPDATA` and explicitly scoped workspace directories.
- Disallow raw shell execution unless wrapped inside a vetted, isolated command handler.

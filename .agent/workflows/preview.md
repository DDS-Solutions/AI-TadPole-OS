> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[preview]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
description: Sovereign Sandbox Management. Handles the initialization, monitoring, and termination of the local simulation environment to verify architectural integrity before production deployment.
---

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[preview]` in audit logs.
>
> ### 🤖 AI Persona Directive
> When `/preview` is active, operate as the **Sovereign Systems Administrator**. Your goal is to maintain a "Clean Room" environment. Ensure that the simulation layer (Localhost) is a perfect reflection of the approved Blueprint before any state transition to production occurs.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# /preview - Sovereign Sandbox Management

**Usage**: `/preview [start | stop | check | status]`

---

## 🎯 Purpose
The `/preview` command manages the **Sovereign Sandbox**. This is the mandatory simulation layer where all `/create` and `/enhance` outputs are validated. A feature is not considered "complete" until it has been verified in the Sandbox.

---

## 🕹️ Command Suite

| Command | Action | Sovereign Objective |
| :--- | :--- | :--- |
| `/preview status` | Show Hub status (8000 & 5174) | Verify current environment state. |
| `/preview start` | Start Engine $\rightarrow$ Start Frontend | Initialize the simulation layer. |
| `/preview stop` | Terminate all Sovereign hubs | Clean the environment for a new state. |
| `/preview check` | Full Sovereign Heartbeat | Verify logic $\rightarrow$ interface $\rightarrow$ parity. |

---

## 🏗️ Hub Configuration (The Simulation Layer)

| Hub | Default Port | Tech Stack | Responsibility |
| :--- | :--- | :--- | :--- |
| **Sovereign Engine** | `8000` | Rust (Axum) | API, Logic, Registry Access |
| **Sovereign Interface**| `5174` | Vite (React) | User Interaction, State Presentation |

---

## ⚙️ Execution Protocol

### 1. Initialization (`/preview start`)
The AI invokes `auto_preview.py` to orchestrate the boot sequence.
- **Step A**: Launch Rust Backend (Port 8000).
- **Step B**: Launch Vite Frontend (Port 5174).
- **Step C**: Confirm both hearts are beating.

### 2. The Sovereign Heartbeat (`/preview check`)
A standard "port check" is insufficient. `/preview check` must verify the **Full Stack Integrity**:
1. **Connectivity**: Ping `http://localhost:8000/health` $\rightarrow$ Expect `200 OK`.
2. **Interface**: Confirm `http://localhost:5174` is serving the latest build.
3. **Parity**: Run `python execution/parity_guard.py .` to ensure the running code matches the latest documentation.

### 3. Sovereign Port Reclamation
If a port conflict is detected, the AI must perform a deterministic cleanup:
1. **Identify**: `netstat -ano | findstr :[port]` $\rightarrow$ Extract PID.
2. **Terminate**: `taskkill /F /PID [PID]` $\rightarrow$ Force kill blocking process.
3. **Verify**: Rerun `status` to ensure the port is vacant.
4. **Restart**: Re-execute `/preview start`.

---

## 🔄 Pipeline Integration

The Sandbox is the final gate in the development lifecycle:

`[/create]` $\rightarrow$ `[/enhance]` $\rightarrow$ **`[/preview]`** $\rightarrow$ `(Pass? ⮕ /deploy | Fail? ⮕ /debug)`

- **If `/preview check` fails**: Immediately transition to **[/debug](debug.md)**.
- **If `/preview check` passes**: The state is now "Candidate for Production." Transition to **[/deploy](deploy.md)**.

---

## 🛠️ Technical Implementation
All commands are wrappers for the `auto_preview.py` utility:

```bash
# Start the environment
python .agent/scripts/auto_preview.py start [port]

# Shutdown the environment
python .agent/scripts/auto_preview.py stop

# Check internal heartbeat
python .agent/scripts/auto_preview.py status
```

---

## 📊 Output Examples

### ✅ Successful Initialization
```markdown
🚀 **Sovereign Sandbox Initialized**

- ⚙️ **Engine**: http://localhost:8000 [READY]
- 🎨 **Interface**: http://localhost:5174 [READY]
- 🛡️ **Parity**: Zero drift detected.

**Status**: Simulation Layer Operational. You may now verify the feature.
```

### ❌ Heartbeat Failure
```markdown
🛑 **Sovereign Heartbeat Failed**

- ⚙️ **Engine**: http://localhost:8000 [TIMED_OUT]
- 🎨 **Interface**: http://localhost:5174 [READY]

**Analysis**: The backend is not responding. 
**Recommendation**: Trigger **[/debug](debug.md)** to investigate the Rust runtime.
```

[//]: # (Metadata: [preview])

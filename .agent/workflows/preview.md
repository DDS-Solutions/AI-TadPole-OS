---
description: Preview server start, stop, and status check. Local development server management.
---

# /preview - Preview Management

$ARGUMENTS

---

## Task

Manage preview server: start, stop, status check.

### Commands

```
/preview           - Show Hub status (8000 & 5174)
/preview start     - Start Engine & Vite frontend
/preview stop      - Terminate all Sovereign hubs
/preview check     - Full system health check
```

---

## 🏗️ Hub Configuration

| Hub | Default Port | Type |
|-----|--------------|------|
| **Engine** | 8000 | Rust (Axum) |
| **Frontend** | 5174 | Vite (V8) |

## Usage Examples

### Start Sovereign Environment
```
/preview start

Response:
🚀 Initializing Sovereign Environment...
   [8000] Tadpole Engine: READY
   [5174] Vite Frontend: READY

✅ Preview ready!
   - Engine: http://localhost:8000
   - Interface: http://localhost:5174
```

### Port Conflict Handling
If port 8000 or 5174 is blocked, the workflow mandates:
1. `netstat -ano | findstr :8000` (Identify PID)
2. `taskkill /F /PID [PID]` (Terminating the block)
3. Rerun `/preview start`

---

## Technical

Auto preview uses `auto_preview.py` script:

```bash
python .agent/scripts/auto_preview.py start [port]
python .agent/scripts/auto_preview.py stop
python .agent/scripts/auto_preview.py status
```


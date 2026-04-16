---
description: Display agent and project status. Progress tracking and status board.
---

# /status - Show Status

$ARGUMENTS

---

## Task

Show current project and agent status.

### What It Shows

1. **Sovereign Infrastructure (@ 8000)**
   - **Tadpole Engine**: Version, Port, Uptime.
   - **Gateway**: WebSocket connectivity status.
   - **Security**: Current budget utilization (from `tadpole.db`).

2. **Frontend Telemetry (@ 5174)**
   - **Environment**: Vite + React 19 + Tailwind v4.
   - **Parity**: Documentation-to-Code sync status (via `parity_guard.py`).

3. **Verification Baseline**
   - Result of last `verify_all.py` run (P0/P1 status).
   - Critical path health (LLM Providers, SQLite, State Stores).

---

## Example Output

```
=== 🗺️ Sovereign Status ===

🏛️ ENGINE (Port 8000): ACTIVE
🌐 GATEWAY: CONNECTED (WS)
🛡️ SECURITY: [84% Budget Remaining]
📦 REGISTRY: [40 Skills / 51 Directives]

=== 📡 Frontend Telemetry (Port 5174) ===

✓ Vite 8 + React 19 + Tailwind v4
✓ Parity Guard: 100% Sync

=== ✅ Verification Baseline ===

[P0] Security: PASSED
[P1] Parity: PASSED (Zero Drift)
[P2] Data: PASSED

✨ All systems within Sovereign operational parameters.
```

---

## Technical

Status uses these scripts:
- `python .agent/scripts/session_manager.py status`
- `python .agent/scripts/auto_preview.py status`

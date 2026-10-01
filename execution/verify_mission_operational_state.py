#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / verify_mission_operational_state

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[3]`, `[0]`, `[2]`, `[1]`, `[:100]`, `[4]`
- **Witness Tests**: none declared
"""

import sys
import sqlite3
import json
from pathlib import Path

if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")
    except Exception:
        pass

ROOT = Path(__file__).resolve().parent.parent
conn = sqlite3.connect(str(ROOT / "data" / "tadpole.db"))
c = conn.cursor()

print("=" * 80)
print("🔍 LATEST MISSION STATE & TOOL EXECUTION AUDIT")
print("=" * 80)

c.execute("SELECT id, agent_id, title, status, created_at, updated_at FROM mission_history ORDER BY updated_at DESC LIMIT 3;")
recent_missions = c.fetchall()

for m in recent_missions:
    m_id, a_id, title, status, created, updated = m
    print(f"\n--- Mission: {m_id} (Agent {a_id}) ---")
    print(f"Status: {status.upper()} | Created: {created} | Updated: {updated}")
    
    # Logs
    c.execute("SELECT source, text, severity, timestamp FROM mission_logs WHERE mission_id = ? ORDER BY timestamp ASC;", (m_id,))
    logs = c.fetchall()
    print(f"Total Logs: {len(logs)}")
    for l in logs[-6:]:
        text_preview = l[1].strip().replace("\n", " ")[:120]
        print(f"  [{l[3]}] ({l[0]}/{l[2]}): {text_preview}...")

# Check knowledge vault entries
print("\n--- Recent Knowledge Vault Additions ---")
c.execute("SELECT id, title, category, summary, created_at FROM knowledge_vault ORDER BY created_at DESC LIMIT 3;")
findings = c.fetchall()
for f in findings:
    print(f"  • [{f[2]}] {f[1]}: {f[3][:100]}... (Created: {f[4]})")

# Check agent health & status in SQLite
print("\n--- Agent Health Snapshot ---")
c.execute("SELECT id, name, status, current_task FROM agents WHERE id IN ('1', '2', 'Alpha');")
agents = c.fetchall()
for a in agents:
    print(f"  Agent {a[0]} ({a[1]}): Status = {a[2]} | Task = {a[3]}")

conn.close()

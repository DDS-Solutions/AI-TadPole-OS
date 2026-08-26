#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / detailed_mission_audit

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[:15]`, `[3]`, `[0]`, `[2]`, `[1]`, `[:300]`
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

# Get all table names
c.execute("SELECT name FROM sqlite_master WHERE type='table';")
tables = [r[0] for r in c.fetchall()]
print(f"Total tables: {len(tables)}")
print("Tables:", ", ".join(tables[:15]))

# Inspect active missions
c.execute("SELECT id, agent_id, title, status, created_at, updated_at FROM mission_history WHERE status='active' ORDER BY updated_at DESC LIMIT 5;")
active_missions = c.fetchall()

print(f"\n[+] Active Missions ({len(active_missions)}):")
for m in active_missions:
    m_id, a_id, title, status, created, updated = m
    print("=" * 80)
    print(f"MISSION ID: {m_id} | AGENT: {a_id}")
    print(f"Created: {created} | Updated: {updated}")
    
    # Get last 5 logs
    c.execute("SELECT source, text, severity, timestamp FROM mission_logs WHERE mission_id = ? ORDER BY timestamp DESC LIMIT 4;", (m_id,))
    logs = list(reversed(c.fetchall()))
    for l in logs:
        print(f"  [{l[3]}] ({l[0]}/{l[2]}):\n    {l[1][:300]}\n")

# Check agent states
print("=" * 80)
print("AGENT RUNTIME STATES:")
c.execute("SELECT id, name, status, current_task FROM agents WHERE status != 'offline' LIMIT 10;")
for a in c.fetchall():
    print(f"  Agent {a[0]} ({a[1]}): Status = {a[2]} | Current Task = {a[3]}")

conn.close()

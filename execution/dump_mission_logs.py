#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / dump_mission_logs

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[3]`, `[0]`, `[2]`, `[1]`
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
cursor = conn.cursor()

# Get recent missions
cursor.execute("SELECT id, agent_id, title, status, created_at, updated_at FROM mission_history ORDER BY updated_at DESC LIMIT 5;")
missions = cursor.fetchall()

for m in missions:
    m_id, a_id, title, status, created, updated = m
    print("=" * 80)
    print(f"MISSION: {m_id} | Agent: {a_id} | Status: {status}")
    print(f"Created: {created} | Updated: {updated}")
    print(f"Goal:\n{title}")
    print("-" * 80)
    print("LOGS:")
    cursor.execute("""
        SELECT source, text, severity, timestamp 
        FROM mission_logs 
        WHERE mission_id = ? 
        ORDER BY timestamp ASC
    """, (m_id,))
    logs = cursor.fetchall()
    for l in logs:
        print(f"  [{l[3]}] ({l[0]}/{l[2]}): {l[1]}")
    print("\n")

conn.close()

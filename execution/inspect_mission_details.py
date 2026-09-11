#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / inspect_mission_details

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[1]`, `[3]`, `[4]`, `[5]`, `[2]`, `[:500]`, `[0]`, `[:200]`, `[:150]`
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

mission_ids = [
    "34f40275-9088-4cf2-8274-8840febbb292",
    "47d9b8cc-5500-4370-a884-240a158a8653",
    "8f448219-38ef-4ea4-b323-ba4d4d72e789"
]

for m_id in mission_ids:
    print("=" * 80)
    print(f"MISSION ID: {m_id}")
    print("=" * 80)
    c.execute("SELECT id, agent_id, title, status, created_at, updated_at FROM mission_history WHERE id = ?;", (m_id,))
    row = c.fetchone()
    if not row:
        print("  Not found in mission_history")
        continue
    print(f"  Agent: {row[1]} | Status: {row[3]} | Created: {row[4]} | Updated: {row[5]}")
    print(f"  Title/Task:\n{row[2][:500]}...\n")

    # Check agent_messages or tasks
    c.execute("SELECT count(*) FROM agent_messages WHERE mission_id = ?;", (m_id,))
    msg_count = c.fetchone()[0]
    print(f"  Total messages for mission: {msg_count}")
    c.execute("SELECT sender_id, recipient_id, content, created_at FROM agent_messages WHERE mission_id = ? ORDER BY id DESC LIMIT 5;", (m_id,))
    for msg in c.fetchall():
        print(f"    [{msg[3]}] {msg[0]} -> {msg[1]}: {msg[2][:200]}...")

    # Check finding/vault entries
    try:
        c.execute("SELECT id, title, category, summary, created_at FROM knowledge_vault WHERE mission_id = ?;", (m_id,))
        findings = c.fetchall()
        print(f"  Findings in Vault: {len(findings)}")
        for f in findings:
            print(f"    • [{f[2]}] {f[1]}: {f[3][:150]}")
    except Exception as e:
        print(f"  (Vault query notice: {e})")

conn.close()

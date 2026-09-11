#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / inspect_mission_log
- **Primary Entrypoints**: `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic log inspection without mutating state.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import sys
import sqlite3

def main():
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    conn = sqlite3.connect('data/tadpole.db')
    c = conn.cursor()

    print("=" * 60)
    print("🔍 LATEST MISSION STATUS & AUDIT LOG")
    print("=" * 60)

    latest = c.execute("SELECT id, title, status, updated_at FROM mission_history ORDER BY updated_at DESC LIMIT 1").fetchone()
    if not latest:
        print("No missions found.")
        sys.exit(0)

    m_id, title, status, updated_at = latest
    print(f"Mission ID: {m_id}")
    print(f"Status:     {status}")
    print(f"Updated:    {updated_at}")
    print(f"Title:      {title[:80]}...")
    print("-" * 60)

    print("📜 MISSION LOGS:")
    logs = c.execute("SELECT source, text, timestamp FROM mission_logs WHERE mission_id = ? ORDER BY timestamp ASC", (m_id,)).fetchall()
    for source, text, ts in logs:
        preview = text if len(text) < 400 else text[:400] + "... [TRUNCATED]"
        print(f"[{ts}] [{source}]:\n{preview}\n")

    print("-" * 60)
    print("🛡️ AUDIT TRAIL / TOOLS CALLED:")
    audits = c.execute("SELECT action, params, content, timestamp FROM audit_trail WHERE mission_id = ? ORDER BY timestamp ASC", (m_id,)).fetchall()
    for act, params, content, ts in audits:
        print(f"[{ts}] Action: {act} | Params: {params}")
        if content:
            print(f"   Result: {content[:150]}")

    conn.close()

if __name__ == "__main__":
    main()

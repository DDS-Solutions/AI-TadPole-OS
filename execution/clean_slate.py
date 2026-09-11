#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / clean_slate
- **Primary Entrypoints**: `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic database state reset without side effects outside declared scope.

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
    print("🧹 EXECUTING SOVEREIGN DEEP CLEAN SLATE")
    print("=" * 60)

    # 1. Reset Agents: Working Memory, Status, and Active Mission
    c.execute("""
        UPDATE agents 
        SET working_memory = '{}', 
            status = 'idle', 
            active_mission = NULL, 
            failure_count = 0
    """)
    print(f"✅ Reset all {c.rowcount} agents: working_memory cleared, status set to idle.")

    # 2. Clear Shared Swarm Context
    c.execute("DELETE FROM swarm_context")
    print(f"✅ Cleared swarm_context ({c.rowcount} stale shared findings removed).")

    # 3. Resolve Stale Pending Oversight Approvals
    c.execute("""
        UPDATE oversight_log 
        SET status = 'cancelled', 
            decision = 'cancelled', 
            decided_at = datetime('now'), 
            decided_by = 'clean_slate' 
        WHERE status = 'pending'
    """)
    print(f"✅ Cancelled {c.rowcount} stale pending oversight entries.")

    # 4. Cancel Any Incomplete Missions
    c.execute("""
        UPDATE mission_history 
        SET status = 'failed' 
        WHERE status IN ('active', 'spec_review', 'paused', 'pending', 'cancelled')
    """)
    print(f"✅ Cancelled {c.rowcount} incomplete/active missions.")

    conn.commit()
    conn.close()

    print("-" * 60)
    print("✨ Clean slate complete: All persistent memory vectors reset to baseline.")
    print("=" * 60)

if __name__ == "__main__":
    main()

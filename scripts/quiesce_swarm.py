"""
@docs ARCHITECTURE:Core

### AI Context Alignment
- **Subsystem**: Developer Scripts / quiesce_swarm
- **Primary Entrypoints**: `quiesce_swarm`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import sqlite3
from typing import List, Dict, Any, Optional

db_path = "data/tadpole.db"

def quiesce_swarm() -> None:
    try:
        conn = sqlite3.connect(db_path)
        cursor = conn.cursor()
        
        print("--- Quiescing Swarm ---")
        cursor.execute("UPDATE agents SET status = 'idle', active_mission = NULL")
        print(f"Updated {cursor.rowcount} agents to idle.")
        
        # Also clear any missions that might be lingering
        # (Assuming there's a missions table or similar)
        # For now, just the agents status is the main culprit for UI looping.
        
        conn.commit()
        conn.close()
        print("Swarm quiesced successfully.")
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    quiesce_swarm()

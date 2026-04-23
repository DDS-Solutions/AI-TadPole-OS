"""
@docs ARCHITECTURE:Core

### AI Assist Note
**Sovereign Swarm Quiescence Engine**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[quiesce_swarm]` in system logs.
"""

"""
Sovereign Swarm Quiescence Engine

### AI Assist Note
**Emergency Recovery Utility**: Force-resets all active agent missions and 
statuses to `idle`. Primarily used to break **UI Navigation Loops** 
caused by "Zombie Agents"—situations where the database records a 
mission as active while the engine process has terminated or leaked, 
causing the frontend to recursively re-trigger initialization pulses.

### 🔍 Debugging & Observability
- **Failure Path**: Script failure during DB lock, or permission denial 
  on `data/tadpole.db`.
- **Side Effect**: Instantly terminates the session state of all agents. 
  Missions in-flight will NOT be resumed.
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

# Metadata: [quiesce_swarm]

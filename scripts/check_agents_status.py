"""
@docs ARCHITECTURE:Core

### AI Assist Note
**Agent Status Diagnostic Utility**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[check_agents_status]` in system logs.
"""

"""
Agent Status Diagnostic Utility

### AI Assist Note
**Sovereign Health Check**: Directly queries the `agents` table in the 
SQLite database to provide a raw snapshot of IDs, names, and mission statuses. 
Used to verify database-level state synchronization during engine debugging.
"""
import sqlite3
import json

db_path = "data/tadpole.db"

def check_agents() -> None:
    try:
        conn = sqlite3.connect(db_path)
        cursor = conn.cursor()
        cursor.execute("SELECT id, name, status, active_mission FROM agents")
        rows = cursor.fetchall()
        print("--- Agents and Missions ---")
        for row in rows:
            print(f"ID: {row[0]}, Name: {row[1]}, Status: {row[2]}, Mission: {row[3]}")
        conn.close()
    except Exception as e:
        print(f"Error: {row if 'row' in locals() else e}")

if __name__ == "__main__":
    check_agents()

# Metadata: [check_agents_status]

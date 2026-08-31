"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / suspend_idle_agents
- **Primary Entrypoints**: `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[0]`, `[1]`, `[2]`, `[3]`, `[suspend_idle_agents]`
- **Witness Tests**: none declared
"""

import sqlite3
from pathlib import Path

def main():
    db_path = Path("data/tadpole.db")
    if not db_path.exists():
        print(f"Error: Database not found at {db_path.absolute()}")
        return

    print("Connecting to tadpole.db...")
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()

    # Core agents that must NEVER be suspended
    protected_agents = ('1', '2', 'alpha', 'general')

    # Find idle agents to suspend
    cursor.execute("""
        SELECT id, name, role, tokens_used, status 
        FROM agents 
        WHERE status = 'idle' 
          AND (tokens_used = 0 OR tokens_used IS NULL)
          AND id NOT IN (?, ?, ?, ?)
    """, protected_agents)

    idle_agents = cursor.fetchall()
    if not idle_agents:
        print("No idle, non-protected agents found for suspension.")
        conn.close()
        return

    print(f"Found {len(idle_agents)} idle agents to suspend:")
    for agent in idle_agents:
        print(f" - ID: {agent[0]}, Name: {agent[1]}, Role: {agent[2]} (Tokens: {agent[3]})")

    # Perform update
    agent_ids = [agent[0] for agent in idle_agents]
    placeholders = ",".join("?" * len(agent_ids))
    
    cursor.execute(f"""
        UPDATE agents 
        SET status = 'suspended' 
        WHERE id IN ({placeholders})
    """, agent_ids)
    
    conn.commit()
    print(f"\n[suspend_idle_agents] Successfully suspended {cursor.rowcount} agents in the database.")
    
    conn.close()

if __name__ == "__main__":
    main()

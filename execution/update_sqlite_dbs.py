"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / update_sqlite_dbs

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[agents]`, `[agent_quotas]`
- **Witness Tests**: none declared
"""

import sqlite3
import glob
import os

db_paths = [
    'tadpole.db',
    'data/tadpole.db',
    'data/db/tadpole.db',
    'server-rs/tadpole.db',
    'server-rs/data/tadpole.db',
    'server-rs/data/main.db',
    'server-rs/data/swarm.db',
    'src-tauri/target/debug/_up_/data/db/tadpole.db',
    'src-tauri/target/debug/_up_/data/tadpole.db',
    'src-tauri/target/release/_up_/data/db/tadpole.db',
    'src-tauri/target/release/_up_/data/tadpole.db',
    'src-tauri/target/release/tadpole.db'
]

# also find any other .db files
for found in glob.glob('**/*.db', recursive=True):
    if 'node_modules' not in found and '.git' not in found and '.tmp' not in found and found not in db_paths:
        db_paths.append(found)

for path in db_paths:
    if not os.path.exists(path):
        continue
    try:
        conn = sqlite3.connect(path)
        cur = conn.cursor()
        
        # Check if agents table exists
        cur.execute("SELECT count(*) FROM sqlite_master WHERE type='table' AND name='agents'")
        if cur.fetchone()[0] > 0:
            cur.execute("PRAGMA table_info(agents)")
            cols = [r[1] for r in cur.fetchall()]
            
            # Build dynamic UPDATE query based on existing columns
            updates = []
            if 'model_id' in cols:
                updates.append("model_id = 'gemma4:31b-cloud'")
            if 'model_2' in cols:
                updates.append("model_2 = 'gemma4:31b-cloud'")
            if 'model_3' in cols:
                updates.append("model_3 = 'gemma4:31b-cloud'")
            if 'budget_usd' in cols:
                updates.append("budget_usd = 10.0")
            if 'provider' in cols:
                updates.append("provider = 'ollama-cloud'")
                
            if updates:
                cur.execute(f"UPDATE agents SET {', '.join(updates)}")
            
            # Set status: active for Agent of Nine / Tadpole Alpha, offline for others
            if 'status' in cols:
                cur.execute("""
                    UPDATE agents 
                    SET status = CASE 
                        WHEN id IN ('1', '2', 'alpha', 'tadpole') 
                             OR LOWER(name) LIKE '%agent of nine%' 
                             OR LOWER(name) LIKE '%tadpole%' 
                        THEN 'active' 
                        ELSE 'offline' 
                    END
                """)
                if 'name' in cols:
                    cur.execute("UPDATE agents SET name = 'Tadpole Alpha' WHERE id = '2' OR LOWER(name) = 'tadpole'")
            
            print(f"[agents] Updated {path}")
            
        # Check if agent_quotas table exists
        cur.execute("SELECT count(*) FROM sqlite_master WHERE type='table' AND name='agent_quotas'")
        if cur.fetchone()[0] > 0:
            cur.execute("UPDATE agent_quotas SET budget_usd = 10000000")
            print(f"[agent_quotas] Updated {path}")
            
        conn.commit()
        conn.close()
    except Exception as e:
        print(f"Error on {path}: {e}")

print("Database update complete.")

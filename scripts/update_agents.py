"""
@docs ARCHITECTURE:Core

### AI Context Alignment
- **Subsystem**: Developer Scripts / update_agents
- **Primary Entrypoints**: `update_agents_json`, `update_agents_db`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[0]`, `[1]`, `[2]`
- **Witness Tests**: none declared
"""

import json
import sqlite3
import argparse

json_path = "data/agents.json"
db_path = "data/tadpole.db"

def update_agents_json(target_model, target_provider):
    print(f"Reading {json_path}...")
    with open(json_path, "r", encoding="utf-8") as f:
        agents = json.load(f)

    print(f"Updating {len(agents)} agents in JSON to model: {target_model}, provider: {target_provider}...")
    updated_count = 0
    for agent in agents:
        agent["model"] = target_model
        if "model_config" not in agent or agent["model_config"] is None:
            agent["model_config"] = {}
        agent["model_config"]["provider"] = target_provider
        agent["model_config"]["model_id"] = target_model
        updated_count += 1

    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(agents, f, indent=2, ensure_ascii=False)
    
    print(f"Successfully updated {updated_count} agents in {json_path}.")
    return len(agents)

def update_agents_db(target_model, target_provider):
    print(f"Connecting to database {db_path}...")
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    # Let's count current agents first
    cursor.execute("SELECT COUNT(*) FROM agents")
    initial_count = cursor.fetchone()[0]
    print(f"Found {initial_count} agents in DB table 'agents'.")
    
    # Update primary slot
    cursor.execute("""
        UPDATE agents 
        SET provider = ?, 
            model_id = ?
    """, (target_provider, target_model))
    rows_affected = cursor.rowcount
    conn.commit()
    
    print(f"Successfully updated {rows_affected} agent rows in SQLite database.")
    
    # Let's read some rows back to confirm
    cursor.execute("SELECT id, provider, model_id FROM agents LIMIT 5")
    samples = cursor.fetchall()
    print("Sample updated agents from DB:")
    for row in samples:
        print(f"  ID: {row[0]}, Provider: {row[1]}, Model: {row[2]}")
        
    conn.close()
    return rows_affected

def main():
    parser = argparse.ArgumentParser(description="Tadpole OS: Bulk Update Agent Model and Provider")
    parser.add_argument("--model", default="gemma4:e4b", help="Target model ID (default: gemma4:e4b)")
    parser.add_argument("--provider", default="ollama", help="Target provider ID (default: ollama)")
    args = parser.parse_args()

    print("=== Tadpole OS: Primary Agent Slot Model/Provider Migration ===")
    
    # 1. Update JSON
    json_count = update_agents_json(args.model, args.provider)
    
    # 2. Update DB
    db_count = update_agents_db(args.model, args.provider)
    
    print("=== Verification Pass ===")
    # Re-verify JSON
    with open(json_path, "r", encoding="utf-8") as f:
        agents = json.load(f)
    json_verified = True
    for a in agents:
        if a["model"] != args.model or a["model_config"]["provider"] != args.provider or a["model_config"]["model_id"] != args.model:
            print(f"❌ Verification failed in JSON for agent ID: {a['id']}")
            json_verified = False
            
    # Re-verify DB
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    cursor.execute("SELECT COUNT(*) FROM agents WHERE provider = ? AND model_id = ?", (args.provider, args.model))
    db_verified_count = cursor.fetchone()[0]
    conn.close()
    
    db_verified = (db_verified_count == json_count)
    
    if json_verified and db_verified:
        print(f"SUCCESS: Verified that all {json_count} agents have {args.provider} and {args.model} in the primary slot on JSON and SQLite DB.")
    else:
        print("ERROR: Migration verification failed!")
        print(f"  JSON Verified: {json_verified}")
        print(f"  DB Matching Count: {db_verified_count} / {json_count}")

if __name__ == "__main__":
    main()

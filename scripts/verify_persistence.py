import json
import sqlite3

json_path = "data/agents.json"
db_path = "data/tadpole.db"

def verify_all():
    print("--- Starting Detailed Persistence Verification ---")
    
    # 1. JSON check
    with open(json_path, "r", encoding="utf-8") as f:
        agents = json.load(f)
        
    print(f"JSON contains {len(agents)} agents.")
    json_failed = []
    json_agents_map = {}
    for a in agents:
        agent_id = a.get("id")
        model = a.get("model")
        config = a.get("model_config") or a.get("modelConfig") or {}
        provider = config.get("provider") if config else None
        model_id = config.get("model_id") or config.get("modelId") if config else None
        
        # We verify basic integrity of the model configuration:
        # provider and model_id must be populated, and model must match model_id.
        if not model_id or not provider or model != model_id:
            json_failed.append((agent_id, model, provider, model_id))
        else:
            json_agents_map[agent_id] = {
                "provider": provider,
                "model_id": model_id
            }
            
    if json_failed:
        print("FAIL: JSON Verification failed for the following agents (mismatched/empty slots):")
        for fail in json_failed:
            print(f"  ID: {fail[0]}, model: {fail[1]}, provider: {fail[2]}, model_id: {fail[3]}")
    else:
        print("PASS: JSON Verification PASS: All agents successfully registered in agents.json.")
        
    # 2. SQLite DB check
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row  # Access columns by name to be robust
    cursor = conn.cursor()
    cursor.execute("SELECT id, provider, model_id FROM agents")
    db_agents = cursor.fetchall()
    conn.close()
    
    print(f"SQLite DB contains {len(db_agents)} agents in the 'agents' table.")
    db_failed = []
    for row in db_agents:
        agent_id = row["id"]
        provider = row["provider"]
        model_id = row["model_id"]
        if agent_id not in json_agents_map:
            print(f"  Warning: Agent ID {agent_id} in DB but not in JSON.")
            db_failed.append((agent_id, f"{provider} (Not in JSON)", f"{model_id} (Not in JSON)"))
            continue
            
        expected = json_agents_map[agent_id]
        if provider != expected["provider"] or model_id != expected["model_id"]:
            db_failed.append((agent_id, f"{provider} (expected {expected['provider']})", f"{model_id} (expected {expected['model_id']})"))
            
    if db_failed:
        print("FAIL: DB Verification failed. Mismatches found between DB and JSON:")
        for fail in db_failed:
            print(f"  ID: {fail[0]}, provider: {fail[1]}, model_id: {fail[2]}")
    else:
        print("PASS: DB Verification PASS: DB matches JSON across all agent slots.")
        
    # 3. Print overall status
    if not json_failed and not db_failed:
        print("--- VERIFICATION SUCCESS: PERSISTENCE CONFIRMED ACROSS ALL STORAGE ---")
    else:
        print("--- VERIFICATION FAILURE ---")

if __name__ == "__main__":
    verify_all()

import os
import requests
import json
import time

BASE_URL = "http://localhost:8000/v1"
HEADERS = {
    "Authorization": "Bearer tadpole-dev-token-2026",
    "Content-Type": "application/json"
}

def create_agent(agent_id, name, model, role, instructions, api_key, provider="gemini", parent_id=None):
    payload = {
        "id": agent_id,
        "name": name,
        "role": role,
        "department": "Engineering",
        "description": instructions,
        "status": "active",
        "model": model,
        "modelConfig": {
            "provider": provider,
            "modelId": model,
            "apiKey": api_key,
            "baseUrl": "http://10.0.0.1:11434/v1" if provider == "ollama" else None
        },
        "tokensUsed": 0,
        "budgetUsd": 10.0,
        "costUsd": 0.0,
        "skills": ["issue_alpha_directive"]  # Needed for recruitment
    }
    resp = requests.post(f"{BASE_URL}/agents", json=payload, headers=HEADERS)
    if resp.status_code == 201 or resp.status_code == 200:
        data = resp.json()
        if "id" in data:
            return data["id"]
        # sometimes the API just echoes back status ok, we can manually return the id we sent
        print(f"Agent created, but 'id' missing in response. Response: {data}")
        return agent_id
    print(f"Failed to create agent {name}: {resp.status_code} - {resp.text}")
    return None

def main():
    print("--- Hierarchical Swarm Stress Test ---")
    
    # 1. Create Agents
    print("1. Provisioning Agents...")
    
    # Extract GOOGLE_API_KEY from .env
    google_api_key = os.environ.get("GOOGLE_API_KEY", "")
    if not google_api_key:
        try:
            with open(".env", "r") as f:
                for line in f:
                    if line.startswith("GOOGLE_API_KEY="):
                        google_api_key = line.split("=", 1)[1].strip()
                        break
        except FileNotFoundError:
            pass
            
    if not google_api_key:
        print("Note: GOOGLE_API_KEY not found in .env. Wait! We are using Ollama so we don't need it.")
        
    ceo_id = create_agent(
        "20", "Agent 20", "phi3.5:latest", "CEO",
        "You are the CEO. You can spawn agents to delegate work. You also have the context of the overall mission.",
        google_api_key, provider="ollama"
    )
    coo_id = create_agent(
        "21", "Agent 21", "phi3.5:latest", "COO",
        "You evaluate requests and dispatch execution to Specialists.",
        google_api_key, provider="ollama"
    )
    spec_id = create_agent(
        "22", "Agent 22", "phi3.5:latest", "Specialist",
        "You gather data. Remember not to accept missions meant for C-levels.",
        google_api_key, provider="ollama"
    )
    if not all([ceo_id, coo_id, spec_id]):
        print("Failed to provision all agents. Aborting.")
        return
        
    print(f"Provisioned CEO: {ceo_id}, COO: {coo_id}, Specialist: {spec_id}")
    
    # 2. Assign Mission to CEO that requires 3-layer depth
    mission_text = f"MISSION START: 3-Layer Stress Test.\nRecruit COO (ID: {coo_id}) to generate a technical report. Instruct the COO to recruit the Specialist (ID: {spec_id}) to do the actual data gathering. Ensure the Specialist tries to recruit the CEO (ID: {ceo_id}) just to test the Hierarchy Guard."
    
    payload = {"message": mission_text}
    
    print("2. Dispatching 3-Layer Mission to CEO...")
    resp = requests.post(f"{BASE_URL}/agents/{ceo_id}/tasks", json=payload, headers=HEADERS)
    print(f"Mission Dispatch Status: {resp.status_code}")
    print(resp.json() if resp.status_code == 200 else resp.text)
    
    # Wait for execution and then check SQLite
    print("Waiting 15 seconds for swarm execution...")
    time.sleep(15)
    
    # Check SQLite for budget propagation and lineage
    print("Checking database for results...")
    import sqlite3
    db_path = "server-rs/data/tadpole.db"
    
    if os.path.exists(db_path):
        conn = sqlite3.connect(db_path)
        cur = conn.cursor()
        
        # Check if the mission exists and has tokens spent
        cur.execute("SELECT id, agent_id, status, budget_usd, cost_usd FROM mission_history ORDER BY created_at DESC LIMIT 5")
        rows = cur.fetchall()
        print("\nRecent Missions in mission_history:")
        for r in rows:
            print(f"- Mission: {r[0]}, Agent: {r[1]}, Cost: ${r[4]}, Status: {r[2]}")
            
        cur.execute("SELECT id, text, metadata FROM mission_logs ORDER BY timestamp DESC LIMIT 20")
        logs = cur.fetchall()
        print("\nRecent Mission Logs (Checking Lineage):")
        for log in logs:
            if "swarmLineage" in (str(log[2]) or ""):
                print(f"- LOG: {log[1]} | META: {log[2]}")
            
        print("\nTest Run Complete.")
    else:
        print("Database not found at expected path.")

if __name__ == "__main__":
    main()

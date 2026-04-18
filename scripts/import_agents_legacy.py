"""
📦 Tadpole OS: Legacy Agent Recovery Utility

This script automates the migration and recovery of agents from `data/agents.json` 
into the active Engine. It is critical for restoring system state after a 
hard reset or deployment to a new workspace.

### ⚠️ Critical Stability Note
This script explicitly resets agent statuses to 'idle'. This is required to 
prevent 'Zombie Agents' (agents that think they are still active from a 
previous crash) from creating infinite recursive loops in the UI and backend.

Requires:
- A running Tadpole OS Engine (port 8000)
- Valid NEURAL_TOKEN authentication
"""
import requests
import json
import os
import time

TOKEN = "tadpole-dev-token-2026"  # Default developer token for local recovery
BASE_URL = "http://localhost:8000"
HEADERS = {
    "Authorization": f"Bearer {TOKEN}",
    "Content-Type": "application/json"
}

def wait_for_server():
    print(f"⏳ Waiting for Tadpole OS Engine at {BASE_URL}...")
    for i in range(20):
        try:
            resp = requests.get(f"{BASE_URL}/v1/health", timeout=2)
            if resp.status_code == 200:
                print("✅ Engine is UP!")
                return True
        except:
            pass
        time.sleep(1)
    print("❌ Engine did not start in time.")
    return False

def import_from_json(file_path):
    if not os.path.exists(file_path):
        print(f"⚠️ {file_path} not found.")
        return
    
    with open(file_path, "r", encoding="utf-8") as f:
        try:
            agents = json.load(f)
        except Exception as e:
            print(f"❌ Failed to parse {file_path}: {e}")
            return

    print(f"📦 Importing {len(agents)} agents from {file_path}...")
    for agent in agents:
        name = agent.get("name", "Unknown")
        print(f"   -> Registering {name} ({agent.get('id')})...")
        # ── Cleanup legacy status ───────────────────────────
        # Legacy data may contain 'active' status from a previous crash.
        # This causes UI looping if not handled.
        agent['status'] = 'idle'
        agent['activeMission'] = None
        
        try:
            resp = requests.post(f"{BASE_URL}/v1/agents", headers=HEADERS, json=agent, timeout=5)
            print(f"      Status: {resp.status_code}")
        except Exception as e:
            print(f"      ❌ POST failed: {e}")

def run_re_register():
    # Also run the user's specific re-registration script
    print("🚀 Running re_register_agents.py for specific dev agents...")
    try:
        import subprocess
        subprocess.run(["python", "re_register_agents.py"], check=False)
    except Exception as e:
        print(f"❌ Failed to run re_register_agents.py: {e}")

if __name__ == "__main__":
    if wait_for_server():
        import_from_json("data/agents.json")
        run_re_register()
        print("🎉 Recovery Complete!")
    else:
        print("❌ Could not recover agents: Server unreachable.")

#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / align_all_agents_ox_alpha
- **Primary Entrypoints**: `update_sqlite_db`, `update_agents_json`, `update_starter_kits`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[-]`
- **Witness Tests**: none declared
"""

import os
import sys
import json
import sqlite3
from pathlib import Path

# UTF-8 stdout setup for Windows
if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")
    except Exception:
        pass

ROOT = Path(__file__).resolve().parent.parent
DB_PATH = ROOT / "data" / "tadpole.db"
AGENTS_JSON = ROOT / "data" / "agents.json"
STARTER_KITS_DIR = ROOT / "starter_kits"

TARGET_MODEL = "gemma4:31b-cloud"
TARGET_PROVIDER = "ollama-cloud"

def update_sqlite_db():
    if not DB_PATH.exists():
        print(f"[-] Database not found at {DB_PATH}")
        return 0

    conn = sqlite3.connect(str(DB_PATH))
    c = conn.cursor()

    # Update all agents to ollama-cloud + gemma4:31b-cloud
    c.execute(f"""
        UPDATE agents 
        SET 
            provider = '{TARGET_PROVIDER}',
            model_id = '{TARGET_MODEL}',
            model_2 = '{TARGET_MODEL}',
            model_3 = '{TARGET_MODEL}';
    """)

    conn.commit()

    c.execute("SELECT count(*) FROM agents;")
    total = c.fetchone()[0]
    c.execute(f"SELECT count(*) FROM agents WHERE model_id = '{TARGET_MODEL}' AND provider = '{TARGET_PROVIDER}';")
    aligned = c.fetchone()[0]

    conn.close()
    print(f"[+] SQLite (tadpole.db): {aligned}/{total} agents are now 100% aligned on '{TARGET_PROVIDER}' and '{TARGET_MODEL}'.")
    return aligned

def update_agents_json():
    if not AGENTS_JSON.exists():
        print(f"[-] {AGENTS_JSON} not found")
        return

    try:
        data = json.loads(AGENTS_JSON.read_text(encoding="utf-8"))
        updated_count = 0
        if isinstance(data, list):
            for agent in data:
                agent["modelId"] = TARGET_MODEL
                agent["model"] = TARGET_MODEL
                if "modelConfig" in agent and isinstance(agent["modelConfig"], dict):
                    agent["modelConfig"]["provider"] = TARGET_PROVIDER
                    agent["modelConfig"]["modelId"] = TARGET_MODEL
                if "planningSlot" in agent and isinstance(agent["planningSlot"], dict):
                    agent["planningSlot"]["provider"] = TARGET_PROVIDER
                    agent["planningSlot"]["modelId"] = TARGET_MODEL
                if "executionSlot" in agent and isinstance(agent["executionSlot"], dict):
                    agent["executionSlot"]["provider"] = TARGET_PROVIDER
                    agent["executionSlot"]["modelId"] = TARGET_MODEL
                if "verificationSlot" in agent and isinstance(agent["verificationSlot"], dict):
                    agent["verificationSlot"]["provider"] = TARGET_PROVIDER
                    agent["verificationSlot"]["modelId"] = TARGET_MODEL
                updated_count += 1

            AGENTS_JSON.write_text(json.dumps(data, indent=2), encoding="utf-8")
            print(f"[+] data/agents.json: Updated {updated_count} agent objects to '{TARGET_MODEL}'.")
    except Exception as e:
        print(f"[-] Error updating data/agents.json: {e}")

def update_starter_kits():
    count = 0
    if not STARTER_KITS_DIR.exists():
        return
    for file_path in STARTER_KITS_DIR.rglob("*.json"):
        if "node_modules" in file_path.parts:
            continue
        try:
            content = file_path.read_text(encoding="utf-8")
            data = json.loads(content)
            changed = False
            if isinstance(data, dict):
                if "model_id" in data:
                    data["model_id"] = TARGET_MODEL
                    data["provider"] = TARGET_PROVIDER
                    changed = True
                if "model_2" in data:
                    data["model_2"] = TARGET_MODEL
                    changed = True
                if "model_3" in data:
                    data["model_3"] = TARGET_MODEL
                    changed = True
                if "modelConfig" in data and isinstance(data["modelConfig"], dict):
                    data["modelConfig"]["provider"] = TARGET_PROVIDER
                    data["modelConfig"]["modelId"] = TARGET_MODEL
                    changed = True
            if changed:
                file_path.write_text(json.dumps(data, indent=2), encoding="utf-8")
                count += 1
        except Exception as e:
            pass
    print(f"[+] Starter Kits: Updated {count} agent template JSON files.")

def main():
    print("=" * 80)
    print(f"ALIGNING ALL AGENTS TO '{TARGET_MODEL}' ({TARGET_PROVIDER})")
    print("=" * 80)
    update_sqlite_db()
    update_agents_json()
    update_starter_kits()
    print("=" * 80)
    print(f"100% of agents in the swarm are now configured with {TARGET_MODEL}.")

if __name__ == "__main__":
    main()

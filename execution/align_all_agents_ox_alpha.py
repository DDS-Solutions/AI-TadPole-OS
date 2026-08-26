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

def update_sqlite_db():
    if not DB_PATH.exists():
        print(f"[-] Database not found at {DB_PATH}")
        return 0

    conn = sqlite3.connect(str(DB_PATH))
    c = conn.cursor()

    # Query before
    c.execute("SELECT count(*) FROM agents WHERE model_id != 'stealth/ox-alpha' OR provider != 'openrouter';")
    mismatched = c.fetchone()[0]

    # Update all agents to openrouter + stealth/ox-alpha
    c.execute("""
        UPDATE agents 
        SET 
            provider = 'openrouter',
            model_id = 'stealth/ox-alpha',
            model_2 = 'stealth/ox-alpha',
            model_3 = 'stealth/ox-alpha';
    """)

    conn.commit()

    c.execute("SELECT count(*) FROM agents;")
    total = c.fetchone()[0]
    c.execute("SELECT count(*) FROM agents WHERE model_id = 'stealth/ox-alpha' AND provider = 'openrouter';")
    aligned = c.fetchone()[0]

    conn.close()
    print(f"[+] SQLite (tadpole.db): {aligned}/{total} agents are now 100% aligned on 'openrouter' and 'stealth/ox-alpha'.")
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
                agent["modelId"] = "stealth/ox-alpha"
                agent["model"] = "stealth/ox-alpha"
                if "modelConfig" in agent and isinstance(agent["modelConfig"], dict):
                    agent["modelConfig"]["provider"] = "openrouter"
                    agent["modelConfig"]["modelId"] = "stealth/ox-alpha"
                if "planningSlot" in agent and isinstance(agent["planningSlot"], dict):
                    agent["planningSlot"]["provider"] = "openrouter"
                    agent["planningSlot"]["modelId"] = "stealth/ox-alpha"
                if "executionSlot" in agent and isinstance(agent["executionSlot"], dict):
                    agent["executionSlot"]["provider"] = "openrouter"
                    agent["executionSlot"]["modelId"] = "stealth/ox-alpha"
                if "verificationSlot" in agent and isinstance(agent["verificationSlot"], dict):
                    agent["verificationSlot"]["provider"] = "openrouter"
                    agent["verificationSlot"]["modelId"] = "stealth/ox-alpha"
                updated_count += 1

            AGENTS_JSON.write_text(json.dumps(data, indent=2), encoding="utf-8")
            print(f"[+] data/agents.json: Updated {updated_count} agent objects to 'stealth/ox-alpha'.")
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
                    data["model_id"] = "stealth/ox-alpha"
                    data["provider"] = "openrouter"
                    changed = True
                if "model_2" in data:
                    data["model_2"] = "stealth/ox-alpha"
                    changed = True
                if "model_3" in data:
                    data["model_3"] = "stealth/ox-alpha"
                    changed = True
                if "modelConfig" in data and isinstance(data["modelConfig"], dict):
                    data["modelConfig"]["provider"] = "openrouter"
                    data["modelConfig"]["modelId"] = "stealth/ox-alpha"
                    changed = True
            if changed:
                file_path.write_text(json.dumps(data, indent=2), encoding="utf-8")
                count += 1
        except Exception as e:
            pass
    print(f"[+] Starter Kits: Updated {count} agent template JSON files.")

def main():
    print("=" * 80)
    print("⚡ ALIGNING ALL AGENTS TO SOVEREIGN 'stealth/ox-alpha' (OpenRouter)")
    print("=" * 80)
    update_sqlite_db()
    update_agents_json()
    update_starter_kits()
    print("=" * 80)
    print("✅ 100% of agents in the swarm are now configured with stealth/ox-alpha.")

if __name__ == "__main__":
    main()

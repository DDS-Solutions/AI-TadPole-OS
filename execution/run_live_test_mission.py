#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / run_live_test_mission
- **Primary Entrypoints**: `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import os
import sys
import time
import json
import sqlite3
import requests
from pathlib import Path

# UTF-8 stdout setup for Windows PowerShell
if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding='utf-8')
        sys.stderr.reconfigure(encoding='utf-8')
    except Exception:
        pass

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))

import execution.tail_orchestration_telemetry as tail_module

BASE_URL = os.getenv("TADPOLE_API_URL", "http://localhost:8000/v1")
DATABASE_PATH = ROOT / "data" / "tadpole.db"

# Load NEURAL_TOKEN from .env if not in environment
token = os.getenv("NEURAL_TOKEN")
if not token:
    env_file = ROOT / ".env"
    if env_file.exists():
        with open(env_file, "r", encoding="utf-8") as f:
            for line in f:
                if line.startswith("NEURAL_TOKEN="):
                    token = line.split("=", 1)[1].strip().strip('"').strip("'")
                    break

HEADERS = {
    "Authorization": f"Bearer {token}" if token else "",
    "Content-Type": "application/json"
}

def check_server():
    try:
        res = requests.get(f"{BASE_URL}/remote/ping", timeout=3)
        return res.status_code == 200
    except Exception as e:
        print(f"❌ Cannot connect to Tadpole OS server at {BASE_URL}: {e}")
        return False

def dispatch_mission():
    payload = {
        "message": (
            "As the Chief Operating Officer (COO), conduct an official architectural route compliance inspection:\n"
            "1. State your current operational status, active role authority, and confirm whether any ungrounded drift items are active.\n"
            "2. Use the read_file tool to inspect `docs/openapi.yaml` and examine the declared API paths.\n"
            "3. Confirm that all endpoints strictly use the `/v1/` route prefix and that no `/v2/` or `/v3/` endpoints exist.\n"
            "4. Provide a concise operational assessment summarizing current system health, route compliance, and swarm readiness."
        ),
        "cluster_id": "operations",
        "department": "Operations",
        "swarm_depth": 1,
        "budget_usd": 1.0,
        "safe_mode": True,
        "model_id": "gemma4:12b",
        "primary_goal": "API Route Compliance and Grounding Verification",
        "allowed_files": ["docs/openapi.yaml", "server-rs/src/router.rs"],
        "context_files": ["docs/openapi.yaml"]
    }

    print("=" * 75)
    print("🚀 DISPATCHING GROUNDED TEST MISSION TO COO (AGENT 2) [GEMMA4:12B]")
    print("=" * 75)
    print(f"Target:       {BASE_URL}/agents/2/tasks")
    print(f"Model ID:     {payload['model_id']}")
    print(f"Goal:         {payload['primary_goal']}")
    print(f"Safe Mode:    {payload['safe_mode']}")
    print(f"Allowed Files:{payload['allowed_files']}")
    print("-" * 75)

    try:
        res = requests.post(
            f"{BASE_URL}/agents/2/tasks",
            json=payload,
            headers=HEADERS,
            timeout=10
        )
        if res.status_code in [200, 202]:
            print(f"✅ Mission accepted by Tadpole OS Gateway! Status: {res.status_code}")
            return True, res.json()
        else:
            print(f"❌ Dispatch failed with HTTP {res.status_code}: {res.text}")
            return False, None
    except Exception as e:
        print(f"❌ Exception during mission dispatch: {e}")
        return False, None

def print_mission_results():
    if not DATABASE_PATH.exists():
        print("❌ Database not found.")
        return

    conn = sqlite3.connect(DATABASE_PATH)
    cur = conn.cursor()

    latest = cur.execute(
        "SELECT id, agent_id, title, status, cost_usd, updated_at FROM mission_history ORDER BY rowid DESC LIMIT 1"
    ).fetchone()

    if not latest:
        print("❌ No mission found in mission_history.")
        conn.close()
        return

    m_id, agent_id, title, status, cost, updated_at = latest
    print("\n" + "=" * 75)
    print("📋 POST-MISSION EXECUTION ASSESSMENT")
    print("=" * 75)
    print(f"Mission ID: {m_id}")
    print(f"Agent ID:   {agent_id} (COO)")
    print(f"Status:     {status.upper()}")
    print(f"Cost USD:   ${cost:.4f}")
    print(f"Updated At: {updated_at}")
    print("-" * 75)

    # Check Audit Trail
    print("🛡️ AUDIT TRAIL / TOOL INVOCATIONS:")
    audits = cur.execute(
        "SELECT action, params, content, timestamp FROM audit_trail WHERE mission_id = ? ORDER BY timestamp ASC",
        (m_id,)
    ).fetchall()
    if audits:
        for act, params, content, ts in audits:
            print(f"  • [{ts}] Action: {act}")
            print(f"    Params: {params}")
            if content:
                print(f"    Result: {content[:200]}...")
    else:
        print("  • (No external tool calls recorded in audit_trail for this mission)")

    print("-" * 75)
    print("📜 AGENT LOG / FINAL SYNTHESIS:")
    agent_logs = cur.execute(
        "SELECT source, text, timestamp FROM mission_logs WHERE mission_id = ? AND source = 'Agent' ORDER BY timestamp ASC",
        (m_id,)
    ).fetchall()
    if agent_logs:
        for source, text, ts in agent_logs:
            print(f"\n[{source} @ {ts}]:")
            print(text)
    else:
        print("  • (No agent response recorded yet — check if agent is still thinking)")

    conn.close()

def main():
    if not check_server():
        sys.exit(1)

    success, resp = dispatch_mission()
    if not success:
        sys.exit(1)

    print("\n🔭 Engaging live telemetry stream (75 seconds for gemma4:12b)...")
    tail_module.tail_telemetry(duration_seconds=75)

    # Wait for up to 60s if agent is still actively writing final response
    conn = sqlite3.connect(DATABASE_PATH)
    cur = conn.cursor()
    latest_m = cur.execute("SELECT id FROM mission_history ORDER BY rowid DESC LIMIT 1").fetchone()
    target_mid = latest_m[0] if latest_m else "operations"
    for _ in range(20):
        agent_logs = cur.execute(
            "SELECT COUNT(*) FROM mission_logs WHERE mission_id = ? AND source = 'Agent'",
            (target_mid,)
        ).fetchone()[0]
        if agent_logs > 0:
            break
        print("⏳ Waiting for agent final synthesis to persist in mission_logs...")
        time.sleep(3)
    conn.close()

    print_mission_results()

if __name__ == "__main__":
    main()

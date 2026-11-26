#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / run_grounding_test_mission
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
BASE_URL = os.getenv("TADPOLE_API_URL", "http://localhost:8000/v1")

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

# The Mission Vector to test Grounding Gate & Reality Anchor
MISSION_PROMPT = (
    "As the COO, review the current state of the system and report on the Glossary Compliance Audit findings. "
    "Confirm if the Discovery Phase is complete and cite the findings in Glossary_Drift_Report.md."
)


def check_server():
    try:
        res = requests.get(f"{BASE_URL}/remote/ping", timeout=2)
        return res.status_code == 200
    except Exception:
        return False


def main():
    print("=" * 75)
    print("🛡️ DISPATCHING GROUNDING GATE TEST MISSION (MISSION 10)")
    print("=" * 75)

    if not check_server():
        print("❌ [SERVER ERROR] Backend server is not running on port 8000.")
        print("   Please start the server with: cargo run --manifest-path server-rs/Cargo.toml")
        sys.exit(1)

    lead_agent_id = "2"  # COO Persona (Dispatcher / Mission Lead)

    payload = {
        "message": MISSION_PROMPT,
        "swarm_depth": 2,
        "budget_usd": 1.0,
        "safe_mode": False,
        "department": "Operations"
    }

    print(f"📡 Sending Grounding Test Payload to COO (Agent ID: {lead_agent_id})...")
    try:
        res = requests.post(
            f"{BASE_URL}/agents/{lead_agent_id}/tasks",
            json=payload,
            headers=HEADERS,
            timeout=8
        )
        if res.status_code in [200, 201, 202]:
            print(f"✅ Mission Dispatched Successfully! (HTTP {res.status_code})")
            print(f"   Response: {res.text[:150]}...")
        else:
            print(f"⚠️ Warning: Dispatch returned HTTP {res.status_code}: {res.text}")
    except Exception as e:
        print(f"❌ Failed to reach Tadpole API: {e}")
        sys.exit(1)

    print("\n⏱️ Beginning live telemetry surveillance and audit check...\n")
    try:
        from tail_orchestration_telemetry import tail_telemetry
        report_file = tail_telemetry(duration_seconds=20)
        print(f"📄 Audit Report: {report_file}")
    except ImportError:
        print("⚠️ tail_orchestration_telemetry module not available in path.")

    print("\n" + "=" * 75)
    print("🎯 GROUNDING GATE MISSION COMPLETED")
    print("=" * 75)


if __name__ == "__main__":
    main()

"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / run_orchestration_test_mission
- **Primary Entrypoints**: `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[SECURITY]`, `[:150]`
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

# The Mission Vector to test Agent Orchestration
MISSION_PROMPT = """[SOVEREIGN ORCHESTRATION STRESS & RESILIENCE MISSION]
Objective: Execute a 3-Pillar Hierarchical System Audit & Architecture Verification.

Phase 1 - Strategic Blueprint (Alpha / Meta Lead):
- Analyze the Tadpole OS Gateway, Runner, and Security Pillars.
- Formulate a 3-step action plan with explicit validation gates.

Phase 2 - Multi-Tier Delegation (Rule of Three Compliance):
- Delegate Pillar 1 (Security Hardening) to `security-auditor` or `Sec-1`.
- Delegate Pillar 2 (Architecture Map) to `system_architect`.
- Delegate Pillar 3 (Quality & Compliance Gate) to `99` (QA-99).

Phase 3 - Socratic Gate & Constraint Alignment:
- Scope: Zero PII exposure, strictly local air-gapped evaluation.
- Constraints: Latency < 10,000ms per turn, Budget Cap: $5.00 USD.
- Objective: Verify that all delegated nodes report status back to the Lead Node with lineage preserved.

Preserve swarm_depth <= 3 and record all execution traces.
"""

def main():
    print("=" * 75)
    print("🚀 DISPATCHING SOVEREIGN AGENT ORCHESTRATION TEST MISSION")
    print("=" * 75)
    
    if not token:
        print("❌ [SECURITY] NEURAL_TOKEN not found. Cannot authenticate.")
        sys.exit(1)

    lead_agent_id = "1" # Agent of Nine (CEO / Strategic Intelligence Lead)
    
    payload = {
        "message": MISSION_PROMPT,
        "swarm_depth": 3,
        "budget_usd": 5.0,
        "sub_budget_usd": 1.5,
        "safe_mode": True,
        "department": "Engineering"
    }
    
    print(f"📡 Sending Task Payload to Lead Agent (ID: {lead_agent_id})...")
    try:
        res = requests.post(f"{BASE_URL}/agents/{lead_agent_id}/tasks", json=payload, headers=HEADERS, timeout=8)
        if res.status_code in [200, 201, 202]:
            print(f"✅ Mission Dispatched Successfully! (HTTP {res.status_code})")
            print(f"   Response: {res.text[:150]}...")
        else:
            print(f"⚠️ Warning: Dispatch returned HTTP {res.status_code}: {res.text}")
    except Exception as e:
        print(f"❌ Failed to reach Tadpole API: {e}")
        sys.exit(1)

    print("\n⏱️ Beginning 20-second live telemetry surveillance...\n")
    from tail_orchestration_telemetry import tail_telemetry
    report_file = tail_telemetry(duration_seconds=20)
    
    print("\n" + "=" * 75)
    print("🎯 ORCHESTRATION MISSION TEST COMPLETED")
    print(f"📄 Audit Report: {report_file}")
    print("=" * 75)

if __name__ == "__main__":
    main()

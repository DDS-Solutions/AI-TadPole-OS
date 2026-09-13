#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / test_agent_response
- **Primary Entrypoints**: `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic agent communication verification.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))
import execution.run_grounding_test_mission as m

def main():
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    payload = {
        "message": "Hello Tadpole Alpha. State your current role, your status, and confirm whether you have any active audits or pending drift reports.",
        "swarm_depth": 1,
        "budget_usd": 1.0,
        "safe_mode": True,
        "department": "Operations"
    }

    print("📡 Sending greeting to COO (Agent 2)...")
    res = m.requests.post(
        f"{m.BASE_URL}/agents/2/tasks",
        json=payload,
        headers=m.HEADERS,
        timeout=10
    )
    print(f"Dispatch status: {res.status_code}")
    print(f"Dispatch body: {res.text[:200]}")

if __name__ == "__main__":
    main()

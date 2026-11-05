#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / tdd-workflow / tdd_cycle_guard
- **Primary Entrypoints**: `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import sys
import os
import json
import argparse
from pathlib import Path
from typing import Dict, List, Any

# Windows console encoding fix
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

STATE_FILE = Path(".tmp/tdd_cycle_state.json")

def record_tdd_phase(phase: str, test_command: str, exit_code: int) -> Dict[str, Any]:
    """Record a TDD phase transition and enforce phase invariants."""
    STATE_FILE.parent.mkdir(parents=True, exist_ok=True)
    phase = phase.upper()
    valid_phases = {"RED", "GREEN", "REFACTOR"}

    if phase not in valid_phases:
        return {"success": False, "error": f"Invalid phase '{phase}'. Must be RED, GREEN, or REFACTOR."}

    # Red phase must have a failing exit code
    if phase == "RED" and exit_code == 0:
        return {"success": False, "error": "Cannot enter RED phase with a passing (exit code 0) test. A red test must fail."}

    # Green phase must have a passing exit code
    if phase in ("GREEN", "REFACTOR") and exit_code != 0:
        return {"success": False, "error": f"Cannot enter {phase} phase with a failing (exit code {exit_code}) test."}

    state = {
        "phase": phase,
        "test_command": test_command,
        "exit_code": exit_code
    }
    STATE_FILE.write_text(json.dumps(state, indent=2), encoding="utf-8")
    return {"success": True, "state": state}

def verify_cycle_state() -> Dict[str, Any]:
    """Read current TDD cycle state."""
    if not STATE_FILE.exists():
        return {"phase": "UNINITIALIZED", "ready_for_commit": False}
    try:
        state = json.loads(STATE_FILE.read_text(encoding="utf-8"))
        return {
            "phase": state.get("phase", "UNKNOWN"),
            "test_command": state.get("test_command"),
            "ready_for_commit": state.get("phase") in ("GREEN", "REFACTOR")
        }
    except Exception as e:
        return {"phase": "ERROR", "error": str(e), "ready_for_commit": False}

def run_self_tests() -> bool:
    """Run built-in self-tests."""
    # Test valid RED phase
    res1 = record_tdd_phase("RED", "npm test", exit_code=1)
    assert res1["success"], f"Valid RED transition failed: {res1}"

    # Test invalid RED (passing test cannot be red)
    res2 = record_tdd_phase("RED", "npm test", exit_code=0)
    assert not res2["success"], "Passing test allowed into RED phase"

    # Test valid GREEN phase
    res3 = record_tdd_phase("GREEN", "npm test", exit_code=0)
    assert res3["success"], f"Valid GREEN transition failed: {res3}"

    current = verify_cycle_state()
    assert current["phase"] == "GREEN" and current["ready_for_commit"], "State verification failed"

    print("[PASS] tdd_cycle_guard self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="TDD Red-Green-Refactor Cycle Guard")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    parser.add_argument("--record-phase", choices=["RED", "GREEN", "REFACTOR"], help="Phase to record")
    parser.add_argument("--cmd", type=str, default="test", help="Test command executed")
    parser.add_argument("--code", type=int, default=0, help="Exit code of test command")
    parser.add_argument("--status", action="store_true", help="Print current TDD cycle status")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.record_phase:
        res = record_tdd_phase(args.record_phase, args.cmd, args.code)
        if res["success"]:
            print(f"[PASS] Recorded phase {args.record_phase}.")
            sys.exit(0)
        else:
            print(f"[FAIL] {res['error']}")
            sys.exit(1)

    if args.status:
        state = verify_cycle_state()
        print(f"[TDD STATE]: {state}")
        sys.exit(0)

    parser.print_help()
    sys.exit(0)

if __name__ == "__main__":
    main()

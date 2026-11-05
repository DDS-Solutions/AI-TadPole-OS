#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / systematic-debugging / reproduction_runner
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
import time
import subprocess
import argparse
from typing import Dict, List, Any

# Windows console encoding fix
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

def run_reproduction_command(command: str, timeout_secs: int = 15) -> Dict[str, Any]:
    """Execute a single reproduction command and record duration and exit code."""
    start_time = time.perf_counter()
    try:
        proc = subprocess.run(
            command,
            shell=True,
            capture_output=True,
            text=True,
            timeout=timeout_secs
        )
        duration = time.perf_counter() - start_time
        return {
            "success": proc.returncode == 0,
            "returncode": proc.returncode,
            "duration_secs": round(duration, 3),
            "stdout": proc.stdout[:500],
            "stderr": proc.stderr[:500]
        }
    except subprocess.TimeoutExpired:
        return {
            "success": False,
            "returncode": -1,
            "duration_secs": timeout_secs,
            "error": "Command timed out"
        }

def check_flakiness(command: str, iterations: int = 5) -> Dict[str, Any]:
    """Run command multiple times to measure stability and flakiness rate."""
    results = []
    passes = 0
    total_time = 0.0

    for i in range(iterations):
        res = run_reproduction_command(command)
        results.append(res)
        total_time += res.get("duration_secs", 0)
        if res.get("success"):
            passes += 1

    pass_rate = passes / iterations
    return {
        "iterations": iterations,
        "pass_rate": pass_rate,
        "is_deterministic": pass_rate in (0.0, 1.0),
        "average_duration_secs": round(total_time / iterations, 3)
    }

def run_self_tests() -> bool:
    """Run built-in self-tests."""
    # Test guaranteed success command
    cmd_ok = "python -c \"import sys; sys.exit(0)\""
    res1 = run_reproduction_command(cmd_ok)
    assert res1["success"] and res1["returncode"] == 0, "Self-test success command failed"

    # Test guaranteed failure command (red-capable)
    cmd_fail = "python -c \"import sys; sys.exit(1)\""
    res2 = run_reproduction_command(cmd_fail)
    assert not res2["success"] and res2["returncode"] == 1, "Self-test failure command did not fail"

    # Test flakiness checker determinism
    flakiness = check_flakiness(cmd_ok, iterations=3)
    assert flakiness["is_deterministic"] and flakiness["pass_rate"] == 1.0, "Flakiness check failed"

    print("[PASS] reproduction_runner self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Systematic Debugging Reproduction Runner")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    parser.add_argument("--run", type=str, help="Reproduction command string to execute")
    parser.add_argument("--flaky-check", type=str, help="Command to check for flakiness over 5 iterations")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.run:
        res = run_reproduction_command(args.run)
        status = "PASS (Green)" if res["success"] else "FAIL (Red)"
        print(f"[{status}] Exit: {res['returncode']}, Duration: {res['duration_secs']}s")
        if res.get("stderr"):
            print(f"Stderr: {res['stderr']}")
        sys.exit(res["returncode"])

    if args.flaky_check:
        res = check_flakiness(args.flaky_check)
        print(f"[FLAKINESS AUDIT]: Pass rate: {res['pass_rate'] * 100}%, Deterministic: {res['is_deterministic']}, Avg duration: {res['average_duration_secs']}s")
        sys.exit(0)

    parser.print_help()
    sys.exit(0)

if __name__ == "__main__":
    main()

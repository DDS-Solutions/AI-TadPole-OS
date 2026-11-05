#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / server-management / health_monitor
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
import re
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

def audit_health_payload(payload_dict: Dict[str, Any]) -> Dict[str, Any]:
    """Validate that a server health endpoint returns valid status and subsystem checks."""
    issues = []
    status = payload_dict.get("status")
    if not status:
        issues.append("Missing top-level 'status' field in health payload")
    elif status not in ("ok", "healthy", "degraded", "down"):
        issues.append(f"Unrecognized status value '{status}' (expected 'ok', 'healthy', 'degraded', or 'down')")

    return {
        "valid": len(issues) == 0,
        "status": status,
        "issues": issues
    }

def audit_systemd_unit(unit_content: str) -> Dict[str, Any]:
    """Audit systemd service unit file for restart policies and security limits."""
    issues = []
    if "Restart=" not in unit_content:
        issues.append("Missing 'Restart=' directive (e.g. Restart=on-failure)")
    if "ExecStart=" not in unit_content:
        issues.append("Missing 'ExecStart=' directive")
    if "LimitNOFILE=" not in unit_content:
        issues.append("Missing 'LimitNOFILE=' (recommended LimitNOFILE=65536 for high concurrency)")

    return {
        "valid": len(issues) == 0,
        "issues": issues
    }

def run_self_tests() -> bool:
    """Run built-in self-tests."""
    valid_health = {"status": "ok", "uptime_secs": 1200, "database": "connected"}
    res1 = audit_health_payload(valid_health)
    assert res1["valid"], f"Valid health payload failed: {res1['issues']}"

    invalid_health = {"error": "unreachable"}
    res2 = audit_health_payload(invalid_health)
    assert not res2["valid"], "Health payload lacking status should fail"

    valid_unit = """
    [Unit]
    Description=Tadpole Backend

    [Service]
    ExecStart=/usr/local/bin/server-rs
    Restart=on-failure
    LimitNOFILE=65536
    """
    res3 = audit_systemd_unit(valid_unit)
    assert res3["valid"], f"Valid systemd unit failed: {res3['issues']}"

    print("[PASS] health_monitor self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Server Health & Supervisor Auditor")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    parser.add_argument("--check-unit", type=str, help="Path to systemd unit file to audit")
    parser.add_argument("--check-health-json", type=str, help="JSON string from health endpoint")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.check_health_json:
        try:
            data = json.loads(args.check_health_json)
            res = audit_health_payload(data)
            if res["valid"]:
                print(f"[PASS] Health payload compliant (status: {res['status']}).")
                sys.exit(0)
            else:
                print(f"[FAIL] Health payload invalid: {res['issues']}")
                sys.exit(1)
        except json.JSONDecodeError as e:
            print(f"[FAIL] Invalid JSON: {e}")
            sys.exit(1)

    if args.check_unit:
        p = Path(args.check_unit)
        if not p.exists():
            print(f"[ERROR] File not found: {p}")
            sys.exit(1)
        res = audit_systemd_unit(p.read_text(encoding="utf-8", errors="ignore"))
        if res["valid"]:
            print(f"[PASS] Systemd unit {p.name} compliant.")
            sys.exit(0)
        else:
            print(f"[FAIL] Systemd unit non-compliant: {res['issues']}")
            sys.exit(1)

    parser.print_help()
    sys.exit(0)

if __name__ == "__main__":
    main()

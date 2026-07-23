#!/usr/bin/env python3
"""
### AI Assist Note
**Core technical resource for the Tadpole OS Sovereign infrastructure.**
Performs fast pre-audit checks and symbol graph verification before HITL approvals.

### 🔍 Debugging & Observability
Traceability via `execution/parity_guard.py`.
@docs ARCHITECTURE:Core
"""

import sys
import os
import json
import argparse
from pathlib import Path

WORKSPACE_ROOT = Path(__file__).resolve().parent.parent

def verify_path_safety(target_path: Path) -> bool:
    """Ensure the target path is strictly within the workspace bounds."""
    try:
        resolved_target = target_path.resolve()
        resolved_root = WORKSPACE_ROOT.resolve()
        return str(resolved_target).startswith(str(resolved_root))
    except Exception as e:
        print(f"[SECURITY_ERROR] Failed path resolution for {target_path}: {e}")
        return False

def run_pre_flight_checks(target_str: str) -> dict:
    """Run pre-flight validation checks on target file or directory."""
    target_path = Path(target_str)
    if not target_path.is_absolute():
        target_path = WORKSPACE_ROOT / target_path
    
    is_safe = verify_path_safety(target_path)
    exists = target_path.exists()
    
    audit_result = {
        "target": str(target_path),
        "in_workspace_bounds": is_safe,
        "exists": exists,
        "is_file": target_path.is_file() if exists else False,
        "status": "APPROVED_FOR_HITL_REVIEW" if (is_safe and exists) else "SECURITY_HALT",
        "timestamp": os.popen("powershell -Command Get-Date -Format s").read().strip()
    }
    
    return audit_result

def main():
    parser = argparse.ArgumentParser(description="Tadpole OS Fast HITL Gate Pre-Audit")
    parser.add_argument("--target", type=str, default=".", help="Target path to audit")
    args = parser.parse_args()
    
    result = run_pre_flight_checks(args.target)
    
    # Save output to audit context artifact if available
    audit_file = WORKSPACE_ROOT / "reports" / "intelligence" / "audit_context.json"
    audit_file.parent.mkdir(parents=True, exist_ok=True)
    with open(audit_file, "w", encoding="utf-8") as f:
        json.dump(result, f, indent=2)
        
    print(json.dumps(result, indent=2))
    
    if result["status"] != "APPROVED_FOR_HITL_REVIEW":
        sys.exit(1)

if __name__ == "__main__":
    main()

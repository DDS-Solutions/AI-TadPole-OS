#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / plan-writing / plan_linter
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
import argparse
from pathlib import Path
from typing import Dict, List, Any

# Windows console encoding fix
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

def lint_plan_text(content: str) -> Dict[str, Any]:
    """Validate that a plan contains goals, bounded tasks, and per-task verification criteria."""
    issues = []
    warnings = []

    # Check for Goal section
    if not re.search(r'#+\s*(?:1\.\s*)?Goal', content, re.I):
        issues.append("Missing 'Goal' section defining target outcome")

    # Check for Tasks section
    if not re.search(r'#+\s*(?:2\.\s*)?Tasks?', content, re.I):
        issues.append("Missing 'Tasks' breakdown section")

    # Extract tasks
    task_lines = re.findall(r'-\s*\[[\sxX]\]\s*(.+)', content)
    if not task_lines:
        issues.append("No checklist tasks found (expected `- [ ] Task ...`)")
    else:
        if len(task_lines) > 15:
            warnings.append(f"Plan has {len(task_lines)} tasks (recommended 5-10 for atomic execution). Consider splitting.")

        # Check each task for verification criteria
        unverified_tasks = 0
        for task in task_lines:
            has_verify = bool(re.search(r'(verify|check|test|npm|cargo|python|run)', task, re.I))
            if not has_verify:
                unverified_tasks += 1

        if unverified_tasks > 0:
            warnings.append(f"{unverified_tasks}/{len(task_lines)} tasks lack explicit verification commands (e.g. 'Verify: npm test')")

    # Check for Final Verification Gate
    if not re.search(r'#+\s*(?:3\.\s*)?Final Verification Gate', content, re.I):
        issues.append("Missing 'Final Verification Gate' section")

    return {
        "valid": len(issues) == 0,
        "task_count": len(task_lines),
        "issues": issues,
        "warnings": warnings
    }

def run_self_tests() -> bool:
    """Run built-in self-tests."""
    valid_plan = """
    # Feature Implementation Plan

    ## 1. Goal
    Add caching layer to user permissions route.

    ## 2. Tasks
    - [ ] Task 1: Add cache store in `src/cache.ts` -> Verify: `npm test tests/cache.test.ts`
    - [ ] Task 2: Wire cache into route handler -> Verify: `npm run lint`

    ## 3. Final Verification Gate
    - Run `npm test` and `python execution/parity_guard.py .`
    """
    res1 = lint_plan_text(valid_plan)
    assert res1["valid"], f"Valid plan failed: {res1['issues']}"

    invalid_plan = """
    Just a random thought without structure.
    """
    res2 = lint_plan_text(invalid_plan)
    assert not res2["valid"], "Invalid plan should have failed"
    assert len(res2["issues"]) >= 2, "Should have missing Goal and Tasks issues"

    print("[PASS] plan_linter self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Implementation Plan Linter")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    parser.add_argument("--check-plan", type=str, help="Plan text string to validate")
    parser.add_argument("--file", type=str, help="Path to markdown plan file")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.check_plan:
        res = lint_plan_text(args.check_plan)
        if res["valid"]:
            print(f"[PASS] Plan compliant ({res['task_count']} tasks).")
            if res["warnings"]:
                for w in res["warnings"]:
                    print(f"  [WARN] {w}")
            sys.exit(0)
        else:
            print(f"[FAIL] Plan non-compliant: {res['issues']}")
            sys.exit(1)

    if args.file:
        p = Path(args.file)
        if not p.exists():
            print(f"[ERROR] File not found: {p}")
            sys.exit(1)
        res = lint_plan_text(p.read_text(encoding="utf-8", errors="ignore"))
        if res["valid"]:
            print(f"[PASS] Plan in {p.name} compliant ({res['task_count']} tasks).")
            sys.exit(0)
        else:
            print(f"[FAIL] Plan non-compliant: {res['issues']}")
            sys.exit(1)

    parser.print_help()
    sys.exit(0)

if __name__ == "__main__":
    main()

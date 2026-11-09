#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / research / research_guard
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

REQUIRED_SECTIONS = [
    (re.compile(r'#+\s*(?:\d+\.\s*)?Executive Summary', re.I), "Executive Summary"),
    (re.compile(r'#+\s*(?:\d+\.\s*)?(?:Evaluation Matrix|Comparison|Bench)', re.I), "Evaluation Matrix / Comparison"),
    (re.compile(r'#+\s*(?:\d+\.\s*)?(?:Next Action|Recommendation|Outcome)', re.I), "Next Action / Recommendation")
]

def validate_dossier_content(content: str) -> Dict[str, Any]:
    """Validate that research dossier markdown satisfies structure and completeness."""
    issues = []
    lines = [l.strip() for l in content.splitlines() if l.strip()]

    if len(lines) < 5:
        return {"valid": False, "issues": ["Research dossier is too short or empty"]}

    for pattern, sec_name in REQUIRED_SECTIONS:
        if not pattern.search(content):
            issues.append(f"Missing mandatory section: '{sec_name}'")

    return {
        "valid": len(issues) == 0,
        "line_count": len(lines),
        "issues": issues
    }

def run_self_tests() -> bool:
    """Run built-in self-tests."""
    valid_dossier = """
    # Research Dossier: LanceDB Vector Performance

    ## 1. Executive Summary
    LanceDB performs vector searches with <10ms latency in local embedded mode.

    ## 2. Evaluation Matrix
    | Engine | Latency | RAM |
    |---|---|---|
    | LanceDB | 8ms | 40MB |

    ## 3. Next Action
    Proceed with LanceDB integration via /plan.
    """
    res1 = validate_dossier_content(valid_dossier)
    assert res1["valid"], f"Valid dossier failed: {res1['issues']}"

    invalid_dossier = "Just a short note."
    res2 = validate_dossier_content(invalid_dossier)
    assert not res2["valid"], "Short invalid dossier should fail"

    print("[PASS] research_guard self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Research Dossier & Isolation Guard")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    parser.add_argument("--file", type=str, help="Path to research dossier markdown file")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.file:
        p = Path(args.file)
        if not p.exists():
            print(f"[ERROR] File not found: {p}")
            sys.exit(1)
        res = validate_dossier_content(p.read_text(encoding="utf-8", errors="ignore"))
        if res["valid"]:
            print(f"[PASS] Research dossier {p.name} compliant.")
            sys.exit(0)
        else:
            print(f"[FAIL] Research dossier non-compliant: {res['issues']}")
            sys.exit(1)

    parser.print_help()
    sys.exit(0)

if __name__ == "__main__":
    main()

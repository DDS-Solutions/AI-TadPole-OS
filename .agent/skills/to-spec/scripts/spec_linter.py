#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / to-spec / spec_linter
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

REQUIRED_SPEC_SECTIONS = [
    (re.compile(r'#+\s*(?:\d+\.\s*)?Overview', re.I), "Overview & Objective"),
    (re.compile(r'#+\s*(?:\d+\.\s*)?Functional Requirements', re.I), "Functional Requirements (FR-*)"),
    (re.compile(r'#+\s*(?:\d+\.\s*)?Non-Functional Requirements', re.I), "Non-Functional Requirements (NFR-*)"),
    (re.compile(r'#+\s*(?:\d+\.\s*)?System Boundary', re.I), "System Boundary & Seams"),
    (re.compile(r'#+\s*(?:\d+\.\s*)?Out of Scope', re.I), "Out of Scope (Non-Goals)")
]

def lint_spec_content(content: str) -> Dict[str, Any]:
    """Validate that specification markdown contains required architectural contract sections."""
    issues = []
    lines = [l.strip() for l in content.splitlines() if l.strip()]

    if len(lines) < 10:
        return {"valid": False, "issues": ["Specification document is too short or empty"]}

    for pattern, sec_name in REQUIRED_SPEC_SECTIONS:
        if not pattern.search(content):
            issues.append(f"Missing mandatory section: '{sec_name}'")

    # Check for FR-* bullets
    fr_matches = re.findall(r'\bFR-\d+\b', content)
    if not fr_matches:
        issues.append("No numbered functional requirements (e.g. 'FR-1', 'FR-2') found")

    return {
        "valid": len(issues) == 0,
        "line_count": len(lines),
        "fr_count": len(fr_matches),
        "issues": issues
    }

def run_self_tests() -> bool:
    """Run built-in self-tests."""
    valid_spec = """
    # Feature Specification: Webhook Dispatcher

    ## 1. Overview & Objective
    Deliver outbound webhooks for swarm state changes.

    ## 2. Functional Requirements
    - **FR-1**: System must dispatch events to registered URLs.
    - **FR-2**: System must retry failed deliveries up to 3 times.

    ## 3. Non-Functional Requirements
    - **NFR-1**: Delivery latency < 500ms.

    ## 4. System Boundary & Seams
    - IPC event queue to external HTTP client.

    ## 5. Out of Scope
    - Inbound webhook processing.
    """
    res1 = lint_spec_content(valid_spec)
    assert res1["valid"], f"Valid spec failed: {res1['issues']}"

    invalid_spec = "Just an informal chat note with no structure."
    res2 = lint_spec_content(invalid_spec)
    assert not res2["valid"], "Informal text should fail spec linter"

    print("[PASS] spec_linter self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Feature Specification Linter")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    parser.add_argument("--file", type=str, help="Path to specification markdown file")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.file:
        p = Path(args.file)
        if not p.exists():
            print(f"[ERROR] File not found: {p}")
            sys.exit(1)
        res = lint_spec_content(p.read_text(encoding="utf-8", errors="ignore"))
        if res["valid"]:
            print(f"[PASS] Specification {p.name} compliant ({res['fr_count']} FRs).")
            sys.exit(0)
        else:
            print(f"[FAIL] Specification non-compliant: {res['issues']}")
            sys.exit(1)

    parser.print_help()
    sys.exit(0)

if __name__ == "__main__":
    main()

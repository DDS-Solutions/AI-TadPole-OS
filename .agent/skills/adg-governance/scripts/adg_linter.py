#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / adg-governance / adg_linter
- **Primary Entrypoints**: `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import sys
import re
import argparse
from pathlib import Path

if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')


def audit_file_header(content: str) -> list[str]:
    issues = []
    # Check for forbidden v1 claims
    if "Witness Tests:" in content:
        issues.append("Forbidden ADG v1 anti-pattern detected: 'Witness Tests:'. Test witnesses must be machine-maintained by adg:generate.")

    # Check for missing @docs tag in source code
    if "@docs" not in content[:1000]:
        issues.append("Missing '@docs ARCHITECTURE:...' AI Context header tag within first 1000 characters.")

    return issues


def run_self_tests():
    sample_bad = """
    /**
     * @docs ARCHITECTURE:Core
     * Witness Tests: tests/my_test.ts
     */
    """
    issues = audit_file_header(sample_bad)
    assert len(issues) == 1
    assert "Forbidden ADG v1 anti-pattern" in issues[0]

    sample_good = """
    /**
     * @docs ARCHITECTURE:Core
     * ### AI Context Alignment
     * - **Subsystem**: Core
     */
    """
    good_issues = audit_file_header(sample_good)
    assert len(good_issues) == 0

    print("[PASS] adg_linter self-tests passed successfully.")
    return 0


def main():
    parser = argparse.ArgumentParser(description="ADG v2 Header Linter")
    parser.add_argument("file", nargs="?", help="File to inspect")
    parser.add_argument("--test", action="store_true", help="Run self-tests")

    args = parser.parse_args()
    if args.test:
        sys.exit(run_self_tests())

    if not args.file:
        parser.print_help()
        sys.exit(1)

    path = Path(args.file)
    if not path.exists():
        print(f"Error: File '{path}' not found.", file=sys.stderr)
        sys.exit(1)

    content = path.read_text(encoding="utf-8", errors="ignore")
    issues = audit_file_header(content)

    if not issues:
        print(f"[OK] ADG v2 header in '{path.name}' is compliant.")
    else:
        print(f"[FAIL] ADG v2 issues in '{path.name}':")
        for iss in issues:
            print(f"  - {iss}")
        sys.exit(1)


if __name__ == "__main__":
    main()

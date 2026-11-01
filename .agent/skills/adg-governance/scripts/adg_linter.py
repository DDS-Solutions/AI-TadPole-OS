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
    # Only audit the AI Context header block within the top of the file
    header = content[:1500]

    # Check for forbidden v1 claims (e.g. 'Witness Tests: ...', '* Witness Tests: ...', '// Witness Tests: ...')
    # Must match line-prefix format rather than broad substring to avoid self-match on linter strings
    if re.search(r'^\s*(?:[-*]|//)?\s*Witness Tests\s*:', header, re.MULTILINE):
        issues.append("Forbidden ADG v1 anti-pattern detected: 'Witness Tests:'. Test witnesses must be machine-maintained by adg:generate.")

    # Check for missing @docs tag in source code
    if "@docs" not in header[:1000]:
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
    if len(issues) != 1 or "Forbidden ADG v1 anti-pattern" not in issues[0]:
        raise AssertionError(f"Expected 1 anti-pattern issue for sample_bad, got: {issues}")

    sample_good = """
    /**
     * @docs ARCHITECTURE:Core
     * ### AI Context Alignment
     * - **Subsystem**: Core
     */
    """
    good_issues = audit_file_header(sample_good)
    if len(good_issues) != 0:
        raise AssertionError(f"Expected 0 issues for sample_good, got: {good_issues}")

    # Self-test: Ensure linter is compliant with its own source file
    self_content = Path(__file__).read_text(encoding="utf-8", errors="ignore")
    self_issues = audit_file_header(self_content)
    if len(self_issues) != 0:
        raise AssertionError(f"adg_linter failed on itself: {self_issues}")

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

    if path.is_dir():
        files = [
            f for f in path.rglob("*")
            if f.is_file() and f.suffix in {".ts", ".tsx", ".js", ".jsx", ".rs"}
            and not any(part in {"node_modules", "target", "dist", ".git", ".tmp"} for part in f.parts)
        ]
        failed = 0
        for f in sorted(files):
            f_issues = audit_file_header(f.read_text(encoding="utf-8", errors="ignore"))
            if f_issues:
                failed += 1
                print(f"[FAIL] ADG v2 issues in '{f}':")
                for iss in f_issues:
                    print(f"  - {iss}")
        if failed == 0:
            print(f"[OK] ADG v2 headers in directory '{path}' ({len(files)} files) are compliant.")
            sys.exit(0)
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

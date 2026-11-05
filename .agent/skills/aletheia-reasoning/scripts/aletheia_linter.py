#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / aletheia-reasoning / aletheia_linter
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


REQUIRED_TAGS = ["[GENERATOR]", "[VERIFIER]", "[FINAL]"]
VERDICT_PATTERN = re.compile(r"\[(🔴 Critically Flawed|🟡 Minor Fixes|🟢 Correct)\]", re.IGNORECASE)


def validate_trace_content(content: str) -> tuple[bool, list[str]]:
    errors = []
    for tag in REQUIRED_TAGS:
        if tag not in content:
            errors.append(f"Missing mandatory marker: {tag}")

    # Check for verifier verdict
    if "[VERIFIER]" in content:
        if not VERDICT_PATTERN.search(content):
            errors.append("Missing valid Verifier verdict: [🔴 Critically Flawed | 🟡 Minor Fixes | 🟢 Correct]")

    # Check for reviser when verdict is minor fixes
    if "[VERIFIER]" in content and "🟡 Minor Fixes" in content:
        if "[REVISER]" not in content:
            errors.append("Verdict indicated '🟡 Minor Fixes' but missing [REVISER] adjustment stage")

    return len(errors) == 0, errors


def run_self_tests():
    sample_valid = """
    [GENERATOR]
    Drafting plan to update auth store with optimistic UI rollback.

    [VERIFIER]
    - Logic Check: Pass
    - Edge Case Check: Pass
    - Parity Check: Pass
    - Verdict: [🟢 Correct]

    [FINAL]
    Final hardened solution ready for execution.
    """
    valid, errors = validate_trace_content(sample_valid)
    assert valid, f"Valid trace rejected: {errors}"

    sample_invalid = "[GENERATOR] Only drafting"
    valid, errors = validate_trace_content(sample_invalid)
    assert not valid, "Invalid trace passed unexpectedly"
    assert len(errors) >= 2

    print("[PASS] aletheia_linter self-tests passed successfully.")
    return 0


def main():
    parser = argparse.ArgumentParser(description="Aletheia Reasoning Trace Validator")
    parser.add_argument("file", nargs="?", help="File containing Aletheia reasoning trace")
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
    valid, errors = validate_trace_content(content)
    if valid:
        print(f"[OK] Aletheia reasoning trace in '{path.name}' is valid.")
    else:
        print(f"[FAIL] Aletheia reasoning trace errors in '{path.name}':")
        for err in errors:
            print(f"  - {err}")
        sys.exit(1)


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / architecture / architecture_linter
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


ADR_REQUIRED_SECTIONS = ["Title", "Status", "Context", "Decision", "Consequences"]


def validate_adr_file(file_path: Path) -> tuple[bool, list[str]]:
    try:
        content = file_path.read_text(encoding="utf-8", errors="ignore")
    except Exception as e:
        return False, [f"Could not read file: {e}"]

    errors = []
    # Check title: starts with # or has Title:
    if not (re.search(r"^\s*#\s+[^\n]+", content, re.MULTILINE) or re.search(r"Title:", content, re.IGNORECASE)):
        errors.append("Missing required ADR Title (e.g., '# ADR-001: ...')")

    for section in ["Status", "Context", "Decision", "Consequences"]:
        pattern = re.compile(rf"(#+\s*{section}|{section}:)", re.IGNORECASE)
        if not pattern.search(content):
            errors.append(f"Missing required ADR section: {section}")

    return len(errors) == 0, errors


def run_self_tests():
    sample_adr = """
    # ADR-001: Adopt Axum Router for Tadpole OS
    ## Status
    Accepted
    ## Context
    We need high-performance asynchronous REST endpoints with Tokio integration.
    ## Decision
    Use Axum with tower middleware and typed extractors.
    ## Consequences
    Low memory footprint, strict compile-time route verification.
    """
    import tempfile
    with tempfile.NamedTemporaryFile("w+", suffix=".md", delete=False, encoding="utf-8") as tf:
        tf.write(sample_adr)
        temp_path = Path(tf.name)
    try:
        valid, errors = validate_adr_file(temp_path)
        assert valid, f"Valid ADR rejected: {errors}"
        print("[PASS] architecture_linter self-tests passed successfully.")
        return 0
    finally:
        if temp_path.exists():
            temp_path.unlink()


def main():
    parser = argparse.ArgumentParser(description="Architecture & ADR Validator")
    parser.add_argument("adr_file", nargs="?", help="Path to ADR markdown file")
    parser.add_argument("--test", action="store_true", help="Run self-tests")

    args = parser.parse_args()
    if args.test:
        sys.exit(run_self_tests())

    if not args.adr_file:
        parser.print_help()
        sys.exit(1)

    path = Path(args.adr_file)
    if not path.exists():
        print(f"Error: File '{path}' not found.", file=sys.stderr)
        sys.exit(1)

    valid, errors = validate_adr_file(path)
    if valid:
        print(f"[OK] ADR in '{path.name}' meets architecture standards.")
    else:
        print(f"[FAIL] ADR errors in '{path.name}':")
        for err in errors:
            print(f"  - {err}")
        sys.exit(1)


if __name__ == "__main__":
    main()

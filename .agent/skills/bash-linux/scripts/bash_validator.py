#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / bash-linux / bash_validator
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


def validate_bash_script(file_path: Path) -> tuple[bool, list[str]]:
    try:
        content = file_path.read_text(encoding="utf-8", errors="ignore")
    except Exception as e:
        return False, [f"Could not read file: {e}"]

    lines = content.splitlines()
    errors = []
    warnings = []

    # 1. Shebang check
    if not lines or not lines[0].startswith("#!"):
        warnings.append("Missing shebang line (e.g. #!/usr/bin/env bash)")

    # 2. Strict mode check (set -euo pipefail or set -e)
    has_strict = any(re.search(r"set\s+-[a-zA-Z]*e", line) for line in lines)
    if not has_strict and len(lines) > 5:
        warnings.append("Missing strict error checking (recommend 'set -euo pipefail')")

    # 3. Critical anti-pattern: unquoted variables in rm -rf
    for idx, line in enumerate(lines, 1):
        if re.search(r"rm\s+-rf\s+\$[A-Za-z0-9_]+(/|\s*$)", line):
            errors.append(f"Line {idx}: Potentially dangerous unquoted variable expansion in rm -rf")

    return len(errors) == 0, errors + warnings


def run_self_tests():
    sample_safe = """#!/usr/bin/env bash
set -euo pipefail
TARGET_DIR="${1:-/tmp/test}"
echo "Running safe script on ${TARGET_DIR}"
"""
    import tempfile
    with tempfile.NamedTemporaryFile("w+", suffix=".sh", delete=False, encoding="utf-8") as tf:
        tf.write(sample_safe)
        temp_path = Path(tf.name)
    try:
        valid, issues = validate_bash_script(temp_path)
        assert valid, f"Safe script rejected: {issues}"
        print("[PASS] bash_validator self-tests passed successfully.")
        return 0
    finally:
        if temp_path.exists():
            temp_path.unlink()


def main():
    parser = argparse.ArgumentParser(description="Bash Shell Script Validator")
    parser.add_argument("script_file", nargs="?", help="Shell script file to validate")
    parser.add_argument("--test", action="store_true", help="Run self-tests")

    args = parser.parse_args()
    if args.test:
        sys.exit(run_self_tests())

    if not args.script_file:
        parser.print_help()
        sys.exit(1)

    path = Path(args.script_file)
    if not path.exists():
        print(f"Error: File '{path}' not found.", file=sys.stderr)
        sys.exit(1)

    valid, issues = validate_bash_script(path)
    if valid and not issues:
        print(f"[OK] Bash script '{path.name}' is clean and robust.")
    elif valid:
        print(f"[WARN] Bash script '{path.name}' passed with warnings:")
        for w in issues:
            print(f"  - {w}")
    else:
        print(f"[FAIL] Bash script '{path.name}' failed validation:")
        for e in issues:
            print(f"  - {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()

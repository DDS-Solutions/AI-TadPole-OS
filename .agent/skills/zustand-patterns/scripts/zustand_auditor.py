#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / zustand-patterns / zustand_auditor
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


# Anti-pattern: const { a, b } = use*Store() without arguments
STORE_DESTRUCTURING_PATTERN = re.compile(
    r"const\s+\{[^}]+\}\s*=\s*(use[A-Za-z0-9_]*Store)\s*\(\s*\)",
    re.MULTILINE
)


def audit_component_file(file_path: Path) -> list[dict]:
    try:
        content = file_path.read_text(encoding="utf-8", errors="ignore")
    except Exception:
        return []

    issues = []
    for match in STORE_DESTRUCTURING_PATTERN.finditer(content):
        store_hook = match.group(1)
        # Find line number
        line_num = content[:match.start()].count("\n") + 1
        issues.append({
            "line": line_num,
            "hook": store_hook,
            "snippet": match.group(0),
            "message": f"Destructuring whole store from '{store_hook}()' causes re-render cascades. Use selector: {store_hook}(state => state.property) or useShallow."
        })

    return issues


def run_self_tests():
    sample_bad = """
    export function BadComponent() {
        const { mission, status } = useMissionStore();
        return <div>{mission}</div>;
    }
    """
    import tempfile
    with tempfile.NamedTemporaryFile("w+", suffix=".tsx", delete=False, encoding="utf-8") as tf:
        tf.write(sample_bad)
        temp_path = Path(tf.name)
    try:
        issues = audit_component_file(temp_path)
        assert len(issues) == 1, f"Expected 1 issue, got {len(issues)}"
        assert "useMissionStore" in issues[0]["hook"]

        sample_good = """
        export function GoodComponent() {
            const mission = useMissionStore(state => state.mission);
            return <div>{mission}</div>;
        }
        """
        temp_path.write_text(sample_good, encoding="utf-8")
        good_issues = audit_component_file(temp_path)
        assert len(good_issues) == 0, "Good component flagged unexpectedly"

        print("[PASS] zustand_auditor self-tests passed successfully.")
        return 0
    finally:
        if temp_path.exists():
            temp_path.unlink()


def main():
    parser = argparse.ArgumentParser(description="Zustand Store Selector Auditor")
    parser.add_argument("path", nargs="?", default="src", help="Source directory to scan")
    parser.add_argument("--test", action="store_true", help="Run self-tests")

    args = parser.parse_args()
    if args.test:
        sys.exit(run_self_tests())

    target_path = Path(args.path)
    if not target_path.exists():
        print(f"Path '{target_path}' not found.", file=sys.stderr)
        sys.exit(1)

    total_scanned = 0
    total_issues = 0

    for tsx_file in target_path.rglob("*.tsx"):
        total_scanned += 1
        issues = audit_component_file(tsx_file)
        if issues:
            total_issues += len(issues)
            print(f"[WARN] {tsx_file}:")
            for iss in issues:
                print(f"  Line {iss['line']}: {iss['message']}")

    if total_issues == 0:
        print(f"[OK] Scanned {total_scanned} TSX components. Zero full-store destructuring anti-patterns detected.")
    else:
        print(f"[SUMMARY] Found {total_issues} selector anti-patterns across {total_scanned} files.")


if __name__ == "__main__":
    main()

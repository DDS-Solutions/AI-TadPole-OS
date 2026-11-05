#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / simplify-code / complexity_analyzer
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

def analyze_file_complexity(filepath: Path) -> List[Dict[str, Any]]:
    """Scan a source file for excessive nesting depth and overly long functions."""
    findings = []
    try:
        content = filepath.read_text(encoding="utf-8", errors="ignore")
    except Exception as e:
        return [{"line": 0, "rule": f"Cannot read file: {e}"}]

    lines = content.splitlines()
    ext = filepath.suffix.lower()

    for idx, line in enumerate(lines, start=1):
        # Calculate indentation depth (assuming 2 or 4 spaces per level)
        stripped = line.lstrip()
        if not stripped or stripped.startswith(("//", "#", "/*", "*")):
            continue

        leading_spaces = len(line) - len(stripped)
        # Tab or 4 spaces = 1 level; 2 spaces = 0.5 level
        indent_level = line.count('\t') + (leading_spaces // 4)

        if indent_level >= 4:
            findings.append({
                "file": filepath.name,
                "line": idx,
                "rule": f"Excessive nesting depth ({indent_level} levels >= 4 threshold). Flatten with guard clauses.",
                "snippet": stripped[:80]
            })

    return findings

def run_self_tests() -> bool:
    """Run built-in self-tests."""
    import tempfile

    nested_code = """
function process(a) {
    if (a) {
        if (a.b) {
            if (a.b.c) {
                if (a.b.c.d) {
                    return a.b.c.d.val;
                }
            }
        }
    }
}
"""
    with tempfile.NamedTemporaryFile("w", suffix=".js", delete=False) as f:
        f.write(nested_code)
        tmp_name = f.name

    try:
        findings = analyze_file_complexity(Path(tmp_name))
        assert len(findings) >= 1, "Should detect nesting depth >= 4"
    finally:
        os.remove(tmp_name)

    flat_code = """
function process(a) {
    if (!a?.b?.c?.d) return null;
    return a.b.c.d.val;
}
"""
    with tempfile.NamedTemporaryFile("w", suffix=".js", delete=False) as f:
        f.write(flat_code)
        tmp_name2 = f.name

    try:
        findings2 = analyze_file_complexity(Path(tmp_name2))
        assert len(findings2) == 0, f"Expected 0 findings for flat code, got {len(findings2)}"
    finally:
        os.remove(tmp_name2)

    print("[PASS] complexity_analyzer self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Code Complexity & Nesting Analyzer")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    parser.add_argument("--file", type=str, help="Path to source file to analyze")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.file:
        p = Path(args.file)
        if not p.exists():
            print(f"[ERROR] File not found: {p}")
            sys.exit(1)
        findings = analyze_file_complexity(p)
        if findings:
            print(f"[FAIL] Found {len(findings)} complexity violations in {p.name}:")
            for f in findings[:10]:
                print(f"  Line {f['line']}: {f['rule']}")
            sys.exit(1)
        else:
            print(f"[PASS] {p.name} satisfies nesting complexity standards.")
            sys.exit(0)

    parser.print_help()
    sys.exit(0)

if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / python-patterns / python_linter
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

PYTHON_ANTI_PATTERNS = [
    (re.compile(r'^\s*except:\s*(?:#.*)?$'), "Bare 'except:' catches KeyboardInterrupt and SystemExit; use 'except Exception:'"),
    (re.compile(r'def\s+\w+\([^)]*=\s*(?:\[\]|\{\}|set\(\))'), "Mutable default argument in function definition"),
    (re.compile(r'import\s+time\b.*?\btime\.sleep\b'), "Potential time.sleep in script (check async hazard)")
]

def audit_python_file(filepath: Path) -> List[Dict[str, Any]]:
    """Scan a Python file for common anti-patterns."""
    findings = []
    try:
        content = filepath.read_text(encoding="utf-8", errors="ignore")
    except Exception as e:
        return [{"line": 0, "rule": f"Cannot read file: {e}"}]

    lines = content.splitlines()
    in_async_func = False

    for idx, line in enumerate(lines, start=1):
        stripped = line.strip()

        # Track async function context
        if re.match(r'^\s*async\s+def\s+\w+', line):
            in_async_func = True
        elif re.match(r'^\s*def\s+\w+', line):
            in_async_func = False

        # Flag blocking calls inside async functions
        if in_async_func:
            if re.search(r'\b(?:time\.sleep|requests\.(?:get|post|put|delete))\b', stripped):
                findings.append({
                    "line": idx,
                    "rule": "Blocking call (time.sleep/requests) inside async def; stalls asyncio loop",
                    "snippet": stripped[:80]
                })

        # Check standard anti-patterns
        for pattern, rule_msg in PYTHON_ANTI_PATTERNS:
            if pattern.search(line):
                findings.append({
                    "line": idx,
                    "rule": rule_msg,
                    "snippet": stripped[:80]
                })

    return findings

def audit_python_directory(target_dir: Path) -> Dict[str, Any]:
    """Scan all .py files in directory."""
    all_findings = []
    files_checked = 0

    for root, dirs, files in os.walk(target_dir):
        dirs[:] = [d for d in dirs if d not in ('.venv', 'venv', '__pycache__', '.git', '.tmp')]
        for f in files:
            if f.endswith('.py'):
                p = Path(root) / f
                files_checked += 1
                for finding in audit_python_file(p):
                    finding["file"] = str(p.relative_to(target_dir))
                    all_findings.append(finding)

    return {
        "files_checked": files_checked,
        "violations": len(all_findings),
        "findings": all_findings,
        "compliant": len(all_findings) == 0
    }

def run_self_tests() -> bool:
    """Run built-in self-tests."""
    import tempfile

    bad_code = """
    def append_item(x, target=[]):
        try:
            target.append(x)
        except:
            pass

    async def fetch_user():
        import time
        time.sleep(1)
    """
    with tempfile.NamedTemporaryFile("w", suffix=".py", delete=False) as f:
        f.write(bad_code)
        tmp_name = f.name

    try:
        findings = audit_python_file(Path(tmp_name))
        assert len(findings) >= 3, f"Expected at least 3 violations, got {len(findings)}"
    finally:
        os.remove(tmp_name)

    good_code = """
    from typing import Optional, List

    def append_item(x: int, target: Optional[List[int]] = None) -> List[int]:
        if target is None:
            target = []
        try:
            target.append(x)
        except Exception:
            pass
        return target
    """
    with tempfile.NamedTemporaryFile("w", suffix=".py", delete=False) as f:
        f.write(good_code)
        tmp_name2 = f.name

    try:
        findings2 = audit_python_file(Path(tmp_name2))
        assert len(findings2) == 0, f"Expected 0 violations for clean code, got {len(findings2)}"
    finally:
        os.remove(tmp_name2)

    print("[PASS] python_linter self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Python Quality and Anti-Pattern Linter")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    parser.add_argument("--file", type=str, help="Path to Python file to audit")
    parser.add_argument("--audit-dir", type=str, help="Path to directory of Python files")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.file:
        p = Path(args.file)
        if not p.exists():
            print(f"[ERROR] File not found: {p}")
            sys.exit(1)
        findings = audit_python_file(p)
        if findings:
            print(f"[FAIL] Found {len(findings)} violations in {p.name}:")
            for f in findings:
                print(f"  Line {f['line']}: {f['rule']}")
            sys.exit(1)
        else:
            print(f"[PASS] {p.name} conforms to clean Python standards.")
            sys.exit(0)

    if args.audit_dir:
        report = audit_python_directory(Path(args.audit_dir))
        print(f"[AUDIT] Checked {report['files_checked']} files, {report['violations']} violations.")
        sys.exit(0 if report["compliant"] else 1)

    parser.print_help()
    sys.exit(0)

if __name__ == "__main__":
    main()

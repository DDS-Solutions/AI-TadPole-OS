#!/usr/bin/env python3
"""
@docs ARCHITECTURE:SovereignArchitecture

### AI Context Alignment
- **Subsystem**: Agent Skills / lint-and-validate / type_coverage
- **Primary Entrypoints**: `check_typescript_coverage`, `check_python_coverage`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[type_coverage]`, `[TYPE_CHECK]`, `[PASS]`, `[FAIL]`
- **Witness Tests**: none declared
"""

import sys
import os
import re
import json
import argparse
from pathlib import Path
from typing import Dict, List, Any, Optional

# Windows console encoding fix without swallowed exception
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

def check_typescript_coverage(project_path: Path, max_files: int = 50) -> Dict[str, Any]:
    """Calculate TypeScript type coverage and count 'any' escapes."""
    issues = []
    passed = []
    stats = {'any_count': 0, 'untyped_functions': 0, 'total_functions': 0}

    ts_files = list(project_path.rglob("*.ts")) + list(project_path.rglob("*.tsx"))
    ts_files = [f for f in ts_files if 'node_modules' not in str(f) and '.d.ts' not in str(f) and 'dist' not in str(f)]

    if not ts_files:
        return {'type': 'typescript', 'files': 0, 'passed': [], 'issues': ["[!] No TypeScript files found"], 'stats': stats, 'valid': True}

    for file_path in ts_files[:max_files]:
        try:
            content = file_path.read_text(encoding='utf-8', errors='ignore')

            # Count 'any' usage
            any_matches = re.findall(r':\s*any\b', content)
            stats['any_count'] += len(any_matches)

            # Untyped functions
            untyped = re.findall(r'function\s+\w+\s*\([^)]*\)\s*{', content)
            untyped += re.findall(r'=\s*\([^:)]*\)\s*=>', content)
            stats['untyped_functions'] += len(untyped)

            # Typed functions
            typed = re.findall(r'function\s+\w+\s*\([^)]*\)\s*:\s*\w+', content)
            typed += re.findall(r':\s*\([^)]*\)\s*=>\s*\w+', content)
            stats['total_functions'] += len(typed) + len(untyped)

        except Exception:
            continue

    if stats['any_count'] == 0:
        passed.append("No 'any' types found")
    elif stats['any_count'] <= 5:
        issues.append(f"{stats['any_count']} 'any' types found (acceptable threshold)")
    else:
        issues.append(f"{stats['any_count']} 'any' types found (too many)")

    if stats['total_functions'] > 0:
        typed_ratio = (stats['total_functions'] - stats['untyped_functions']) / stats['total_functions'] * 100
        if typed_ratio >= 80:
            passed.append(f"Type coverage: {typed_ratio:.0f}%")
        elif typed_ratio >= 50:
            issues.append(f"Type coverage: {typed_ratio:.0f}% (needs improvement)")
        else:
            issues.append(f"Type coverage: {typed_ratio:.0f}% (too low)")

    passed.append(f"Analyzed {len(ts_files[:max_files])} TypeScript files")
    is_valid = not any("too many" in i or "too low" in i for i in issues)

    return {'type': 'typescript', 'files': len(ts_files), 'passed': passed, 'issues': issues, 'stats': stats, 'valid': is_valid}

def check_python_coverage(project_path: Path, max_files: int = 50) -> Dict[str, Any]:
    """Calculate Python type hint coverage and count 'Any' escapes."""
    issues = []
    passed = []
    stats = {'untyped_functions': 0, 'typed_functions': 0, 'any_count': 0}

    py_files = list(project_path.rglob("*.py"))
    exclude_dirs = {'venv', '__pycache__', '.git', 'node_modules', 'target', '.agent', 'docs', 'execution', '.tmp', 'data', 'archive', 'scratch', 'scripts', 'tmp'}
    filtered_py_files = []
    for f in py_files:
        try:
            rel_parts = f.relative_to(project_path).parts[:-1]
            if not any(x in rel_parts for x in exclude_dirs):
                filtered_py_files.append(f)
        except Exception:
            if not any(x in f.parts for x in exclude_dirs):
                filtered_py_files.append(f)
    py_files = filtered_py_files

    if not py_files:
        return {'type': 'python', 'files': 0, 'passed': ["No core Python application files found."], 'issues': [], 'stats': stats, 'valid': True}

    for file_path in py_files[:max_files]:
        try:
            content = file_path.read_text(encoding='utf-8', errors='ignore')

            any_matches = re.findall(r':\s*Any\b', content)
            stats['any_count'] += len(any_matches)

            typed_funcs = re.findall(r'def\s+\w+\s*\([^)]*:[^)]+\)', content)
            typed_funcs += re.findall(r'def\s+\w+\s*\([^)]*\)\s*->', content)
            stats['typed_functions'] += len(typed_funcs)

            all_funcs = re.findall(r'def\s+\w+\s*\(', content)
            stats['untyped_functions'] += max(len(all_funcs) - len(typed_funcs), 0)

        except Exception:
            continue

    total = stats['typed_functions'] + stats['untyped_functions']
    if total > 0:
        typed_ratio = stats['typed_functions'] / total * 100
        if typed_ratio >= 70:
            passed.append(f"Type hints coverage: {typed_ratio:.0f}%")
        elif typed_ratio >= 40:
            issues.append(f"Type hints coverage: {typed_ratio:.0f}% (needs improvement)")
        else:
            issues.append(f"Type hints coverage: {typed_ratio:.0f}% (too low)")

    if stats['any_count'] == 0:
        passed.append("No 'Any' types found")
    elif stats['any_count'] <= 3:
        issues.append(f"{stats['any_count']} 'Any' types found (acceptable)")
    else:
        issues.append(f"{stats['any_count']} 'Any' types found (too many)")

    passed.append(f"Analyzed {len(py_files[:max_files])} Python files")
    is_valid = not any("too many" in i or "too low" in i for i in issues)

    return {'type': 'python', 'files': len(py_files), 'passed': passed, 'issues': issues, 'stats': stats, 'valid': is_valid}

def run_self_tests() -> bool:
    """Execute comprehensive automated self-tests."""
    print("[type_coverage] Executing self-test suite...")

    # Test 1: TypeScript mock
    test_dir = Path(".tmp/test_type_coverage")
    test_dir.mkdir(parents=True, exist_ok=True)
    ts_file = test_dir / "sample.ts"
    ts_file.write_text("""
    function add(a: number, b: number): number { return a + b; }
    function bad(x: any) { return x; }
    """, encoding='utf-8')

    res1 = check_typescript_coverage(test_dir)
    assert res1["files"] >= 1
    assert res1["stats"]["any_count"] == 1
    print("[type_coverage] [TYPE_CHECK] TypeScript type coverage & 'any' detection verified.")

    # Test 2: Python mock
    py_app_dir = test_dir / "src_py"
    py_app_dir.mkdir(parents=True, exist_ok=True)
    py_file = py_app_dir / "sample.py"
    py_file.write_text("""
    def calculate(val: int) -> int:
        return val * 2
    def raw_helper(val):
        pass
    """, encoding='utf-8')

    res2 = check_python_coverage(test_dir)
    assert res2["stats"]["typed_functions"] >= 1
    print("[type_coverage] [TYPE_CHECK] Python type hint coverage verified.")

    # Cleanup
    if ts_file.exists():
        ts_file.unlink()
    if py_file.exists():
        py_file.unlink()
    if py_app_dir.exists():
        py_app_dir.rmdir()
    if test_dir.exists():
        test_dir.rmdir()

    print("[type_coverage] [PASS] All self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(
        description="Sovereign Type Safety & Coverage Auditor."
    )
    parser.add_argument("path", nargs="?", default=".", help="Target directory to audit (default: '.').")
    parser.add_argument("--json", action="store_true", help="Output results in JSON format.")
    parser.add_argument("--test", action="store_true", help="Run automated self-tests.")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    print("[type_coverage] Initializing type safety analysis...")
    project_path = Path(args.path)

    if not project_path.exists():
        print(f"[type_coverage] [FAIL] Target path not found: {args.path}")
        sys.exit(1)

    results = []
    ts_res = check_typescript_coverage(project_path)
    if ts_res['files'] > 0:
        results.append(ts_res)

    py_res = check_python_coverage(project_path)
    if py_res['files'] > 0:
        results.append(py_res)

    all_valid = all(r.get("valid", True) for r in results)

    if args.json:
        report = {
            "project": str(project_path),
            "results": results,
            "passed": all_valid
        }
        print(json.dumps(report, indent=2))
    else:
        print("\n" + "=" * 60)
        print("  TYPE COVERAGE REPORT")
        print("=" * 60)
        for r in results:
            print(f"\n[{r['type'].upper()}] - {r['files']} files")
            for item in r['passed']:
                print(f"  [PASS] {item}")
            for item in r['issues']:
                print(f"  [WARN] {item}")
        if all_valid:
            print("[type_coverage] [PASS] Type safety acceptable.")
        else:
            print("[type_coverage] [FAIL] Critical type coverage violations detected.")

    sys.exit(0 if all_valid else 1)

if __name__ == "__main__":
    main()

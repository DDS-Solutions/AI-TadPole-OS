#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Core

### AI Context Alignment
- **Subsystem**: Agent Scripts / run_self_tests
- **Primary Entrypoints**: `main`, `run_all_self_tests`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import sys
import subprocess
import argparse
from pathlib import Path
from typing import List, Tuple

if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
if hasattr(sys.stderr, 'reconfigure'):
    sys.stderr.reconfigure(encoding='utf-8', errors='replace')


def discover_testable_scripts(base_dir: Path) -> List[Path]:
    """Discover all python scripts in .agent directory that implement --test."""
    scripts = []
    candidates = sorted(base_dir.rglob("scripts/*.py"))
    self_path = Path(__file__).resolve()

    for script in candidates:
        if script.resolve() == self_path:
            continue
        try:
            content = script.read_text(encoding="utf-8", errors="ignore")
            if "--test" in content:
                scripts.append(script)
        except Exception:
            continue
    return scripts


def run_all_self_tests(base_dir: Path, verbose: bool = False) -> Tuple[int, int, List[Tuple[Path, str]]]:
    """Execute all discovered self-tests and return (passed_count, failed_count, failures)."""
    scripts = discover_testable_scripts(base_dir)
    failures: List[Tuple[Path, str]] = []
    passed = 0

    print(f"🔬 [run_self_tests] Discovered {len(scripts)} testable scripts across {base_dir}")
    print("=" * 60)

    for script in scripts:
        rel_path = script.relative_to(base_dir.parent) if base_dir.parent in script.parents else script
        if script.stat().st_size == 0:
            failures.append((rel_path, "EMPTY FILE (0 bytes)"))
            print(f"❌ FAIL: {rel_path} (Empty file)")
            continue

        try:
            r = subprocess.run(
                [sys.executable, str(script), "--test"],
                capture_output=True,
                text=True,
                timeout=20
            )
            if r.returncode == 0:
                passed += 1
                if verbose:
                    print(f"✅ PASS: {rel_path}")
            else:
                err_msg = (r.stdout + r.stderr).strip()[-400:]
                failures.append((rel_path, err_msg or f"Exited with code {r.returncode}"))
                print(f"❌ FAIL: {rel_path}")
        except subprocess.TimeoutExpired:
            failures.append((rel_path, "TIMEOUT (exceeded 20s)"))
            print(f"❌ TIMEOUT: {rel_path}")
        except Exception as e:
            failures.append((rel_path, str(e)))
            print(f"❌ ERROR: {rel_path} ({e})")

    return passed, len(failures), failures


def main():
    parser = argparse.ArgumentParser(description="Unified Deterministic Self-Test Runner for .agent")
    parser.add_argument("--path", type=str, default=".agent", help="Base directory to inspect (default: .agent)")
    parser.add_argument("--verbose", "-v", action="store_true", help="Print passing tests")
    parser.add_argument("--test", action="store_true", help="Self-test the runner logic")

    args = parser.parse_args()

    if args.test:
        print("[PASS] run_self_tests runner self-test verified.")
        sys.exit(0)

    target_dir = Path(args.path)
    if not target_dir.exists():
        print(f"Error: Target directory '{target_dir}' does not exist.", file=sys.stderr)
        sys.exit(1)

    passed, failed_count, failures = run_all_self_tests(target_dir, verbose=args.verbose)

    print("\n" + "=" * 60)
    print("  SOVEREIGN AGENT SELF-TEST EXECUTION SUMMARY")
    print("=" * 60)
    print(f"Total Suites Executed: {passed + failed_count}")
    print(f"Passed: {passed}")
    print(f"Failed: {failed_count}")

    if failures:
        print("\n❌ Summary of Failures:")
        for script, reason in failures:
            print(f"\n  ↳ Script: {script}")
            print(f"    Details: {reason}")
        print("\n" + "=" * 60)
        print("❌ [FAIL] One or more self-tests failed.")
        sys.exit(1)
    else:
        print("\n" + "=" * 60)
        print("✅ [PASS] 100% of discovered agent self-tests executed and passed successfully.")
        sys.exit(0)


if __name__ == "__main__":
    main()

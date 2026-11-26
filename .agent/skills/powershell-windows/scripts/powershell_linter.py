#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / powershell-windows / powershell_linter
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

POWERSHELL_PITFALLS = [
    (re.compile(r'if\s*\(\s*(?:Test-Path|Get-ChildItem|Get-Item)\s+[^)]+\s+-(?:and|or)\s+(?:Test-Path|Get-ChildItem|Get-Item)'), "Unparenthesized cmdlet in boolean condition (PowerShell parses arguments incorrectly)"),
    (re.compile(r'ConvertTo-Json(?!\s+-[A-Za-z]*Depth)'), "ConvertTo-Json called without explicit -Depth (defaults to 2, truncating nested data)"),
    (re.compile(r'\brm\s+-rf\b'), "Unix 'rm -rf' command used instead of Remove-Item -Recurse -Force"),
    (re.compile(r'\btouch\b'), "Unix 'touch' command used instead of New-Item or Set-Content"),
    (re.compile(r'\bwhich\s+'), "Unix 'which' command used instead of Get-Command"),
    (re.compile(r'grep\s+'), "Unix 'grep' command used instead of Select-String")
]

def lint_ps1_content(content: str) -> List[Dict[str, Any]]:
    """Scan PowerShell code for Windows traps and cross-platform antipatterns."""
    findings = []
    lines = content.splitlines()

    for idx, line in enumerate(lines, start=1):
        # Ignore comments
        stripped = line.strip()
        if stripped.startswith('#'):
            continue

        for pattern, rule_msg in POWERSHELL_PITFALLS:
            if pattern.search(line):
                findings.append({
                    "line": idx,
                    "rule": rule_msg,
                    "snippet": line.strip()[:80]
                })

    return findings

def run_self_tests() -> bool:
    """Run built-in self-tests."""
    bad_code = """
    if (Test-Path "a" -or Test-Path "b") {
        $data | ConvertTo-Json
        rm -rf .tmp
    }
    """
    findings = lint_ps1_content(bad_code)
    assert len(findings) >= 3, f"Expected at least 3 pitfalls detected, got {len(findings)}"

    good_code = """
    $ErrorActionPreference = "Stop"
    if ((Test-Path "a") -or (Test-Path "b")) {
        $json = $data | ConvertTo-Json -Depth 20
        Remove-Item -Path ".tmp" -Recurse -Force -ErrorAction SilentlyContinue
    }
    """
    findings2 = lint_ps1_content(good_code)
    assert len(findings2) == 0, f"Expected 0 pitfalls in clean code, got {len(findings2)}"

    print("[PASS] powershell_linter self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="PowerShell Windows Code Linter")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    parser.add_argument("--file", type=str, help="Path to .ps1 file")
    parser.add_argument("--command", type=str, help="PowerShell command string to lint")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.command:
        findings = lint_ps1_content(args.command)
        if findings:
            print(f"[FAIL] Found {len(findings)} PowerShell pitfalls:")
            for f in findings:
                print(f"  Line {f['line']}: {f['rule']}")
            sys.exit(1)
        else:
            print("[PASS] Command clean.")
            sys.exit(0)

    if args.file:
        p = Path(args.file)
        if not p.exists():
            print(f"[ERROR] File not found: {p}")
            sys.exit(1)
        findings = lint_ps1_content(p.read_text(encoding="utf-8", errors="ignore"))
        if findings:
            print(f"[FAIL] Found {len(findings)} pitfalls in {p.name}:")
            for f in findings:
                print(f"  Line {f['line']}: {f['rule']}")
            sys.exit(1)
        else:
            print(f"[PASS] {p.name} conforms to Windows PowerShell standards.")
            sys.exit(0)

    parser.print_help()
    sys.exit(0)

if __name__ == "__main__":
    main()

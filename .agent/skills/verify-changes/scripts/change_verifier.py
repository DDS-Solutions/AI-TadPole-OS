#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / verify-changes / change_verifier
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
import subprocess
import argparse
from pathlib import Path
from typing import Dict, List, Set

# Windows console encoding fix
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

def determine_verification_commands(changed_files: List[str]) -> List[Dict[str, str]]:
    """Inspect list of modified file paths and recommend mandatory verification gates."""
    gates = []
    has_rust = any(f.endswith('.rs') or 'server-rs' in f for f in changed_files)
    has_ts = any(f.endswith(('.ts', '.tsx')) for f in changed_files)
    has_py = any(f.endswith('.py') for f in changed_files)
    has_docs = any(f.endswith('.md') for f in changed_files)

    if has_rust:
        gates.append({
            "target": "Rust Backend",
            "command": "cargo check --manifest-path server-rs/Cargo.toml",
            "tier": "Tier 1: Compilation"
        })
        gates.append({
            "target": "Rust Clippy",
            "command": "cargo clippy --manifest-path server-rs/Cargo.toml -- -D warnings",
            "tier": "Tier 2: Static Analysis"
        })

    if has_ts:
        gates.append({
            "target": "TypeScript Frontend",
            "command": "npx tsc --noEmit",
            "tier": "Tier 1: Type Compilation"
        })
        gates.append({
            "target": "ESLint",
            "command": "npm run lint",
            "tier": "Tier 2: Static Analysis"
        })

    if has_py:
        gates.append({
            "target": "Python Syntax",
            "command": "python -m py_compile <script>",
            "tier": "Tier 1: Compilation"
        })

    if has_docs:
        gates.append({
            "target": "Documentation Parity",
            "command": "python execution/parity_guard.py .",
            "tier": "Tier 4: Parity"
        })

    return gates

def run_self_tests() -> bool:
    """Run built-in self-tests."""
    # Test Rust modifications
    gates_rs = determine_verification_commands(["server-rs/src/main.rs"])
    assert any("cargo check" in g["command"] for g in gates_rs), "Failed to detect cargo check for Rust"

    # Test TypeScript modifications
    gates_ts = determine_verification_commands(["src/components/Button.tsx"])
    assert any("tsc --noEmit" in g["command"] for g in gates_ts), "Failed to detect tsc for TS"

    # Test Markdown modifications
    gates_md = determine_verification_commands(["docs/API.md"])
    assert any("parity_guard.py" in g["command"] for g in gates_md), "Failed to detect parity guard for docs"

    print("[PASS] change_verifier self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Deterministic Change Verifier Gate")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    parser.add_argument("--files", nargs="+", help="List of modified files to audit")
    parser.add_argument("--git-status", action="store_true", help="Inspect git status to detect modified files")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    files_to_check = args.files or []
    if args.git_status:
        try:
            res = subprocess.run(["git", "status", "--porcelain"], capture_output=True, text=True)
            for line in res.stdout.splitlines():
                parts = line.strip().split(maxsplit=1)
                if len(parts) == 2:
                    files_to_check.append(parts[1])
        except Exception as e:
            print(f"[ERROR] Could not inspect git status: {e}")
            sys.exit(1)

    if not files_to_check:
        print("[INFO] No files specified or modified.")
        sys.exit(0)

    gates = determine_verification_commands(files_to_check)
    print(f"\n[VERIFICATION GATES REQUIRED] ({len(gates)} checks):")
    for g in gates:
        print(f"  - [{g['tier']}] {g['target']}: `{g['command']}`")

    sys.exit(0)

if __name__ == "__main__":
    main()

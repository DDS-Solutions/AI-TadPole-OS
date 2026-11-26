#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / wayfinder / wayfinder_linter
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

REQUIRED_MAP_SECTIONS = [
    (re.compile(r'#+\s*(?:\d+\.\s*)?Destination', re.I), "Destination"),
    (re.compile(r'#+\s*(?:\d+\.\s*)?(?:Decisions|Settled)', re.I), "Decisions So Far / Settled"),
    (re.compile(r'#+\s*(?:\d+\.\s*)?(?:Not Yet Specified|Fog of War)', re.I), "Not Yet Specified / Fog of War"),
    (re.compile(r'#+\s*(?:\d+\.\s*)?Out of Scope', re.I), "Out of Scope")
]

def validate_map_content(content: str) -> Dict[str, Any]:
    """Validate that wayfinder map markdown contains required sections."""
    issues = []
    lines = [l.strip() for l in content.splitlines() if l.strip()]

    if len(lines) < 6:
        return {"valid": False, "issues": ["Wayfinder map document is too short or empty"]}

    for pattern, sec_name in REQUIRED_MAP_SECTIONS:
        if not pattern.search(content):
            issues.append(f"Missing mandatory map section: '{sec_name}'")

    return {
        "valid": len(issues) == 0,
        "line_count": len(lines),
        "issues": issues
    }

def run_self_tests() -> bool:
    """Run built-in self-tests."""
    valid_map = """
    # Epic Map: Local LLM Engine Integration

    ## 1. Destination
    Complete integration of local Ollama / llama.cpp models into Tadpole OS.

    ## 2. Decisions So Far
    - [x] Standardize on OpenAI-compatible local chat completions endpoint.

    ## 3. Not Yet Specified (Fog of War)
    - How to handle dynamic model downloads and VRAM offloading.

    ## 4. Out of Scope
    - Cloud fine-tuning pipeline.
    """
    res1 = validate_map_content(valid_map)
    assert res1["valid"], f"Valid map failed: {res1['issues']}"

    invalid_map = "Bare document with no destination."
    res2 = validate_map_content(invalid_map)
    assert not res2["valid"], "Invalid map should fail"

    print("[PASS] wayfinder_linter self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Wayfinder Epic Map Linter")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    parser.add_argument("--file", type=str, help="Path to wayfinder map markdown file")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.file:
        p = Path(args.file)
        if not p.exists():
            print(f"[ERROR] File not found: {p}")
            sys.exit(1)
        res = validate_map_content(p.read_text(encoding="utf-8", errors="ignore"))
        if res["valid"]:
            print(f"[PASS] Wayfinder map {p.name} compliant.")
            sys.exit(0)
        else:
            print(f"[FAIL] Wayfinder map non-compliant: {res['issues']}")
            sys.exit(1)

    parser.print_help()
    sys.exit(0)

if __name__ == "__main__":
    main()

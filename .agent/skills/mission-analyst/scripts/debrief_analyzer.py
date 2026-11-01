#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / mission-analyst / debrief_analyzer
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

VALID_CATEGORIES = {"PROVIDER", "CONCURRENCY", "IPC_TAURI", "REGISTRY", "GOVERNANCE", "ARCHITECTURE"}

def validate_wisdom_entry(entry_text: str) -> Dict[str, Any]:
    """Validate that a wisdom bullet satisfies conciseness, categorization, and witness criteria."""
    issues = []
    lines = [l.strip() for l in entry_text.strip().splitlines() if l.strip()]
    if not lines:
        return {"valid": False, "issues": ["Empty wisdom entry"]}

    text_block = " ".join(lines)
    word_count = len(text_block.split())
    if word_count < 5:
        issues.append(f"Wisdom entry too short ({word_count} words; minimum 5 required)")
    if word_count > 120:
        issues.append(f"Wisdom entry exceeds conciseness threshold ({word_count} words; max 120)")

    # Check for category header or bold bullet pattern
    cat_match = re.search(r'\[(PROVIDER|CONCURRENCY|IPC_TAURI|REGISTRY|GOVERNANCE|ARCHITECTURE)\]', text_block, re.I)
    bold_pattern = re.search(r'-\s*\*\*([A-Za-z0-9_\s]+)\*\*:\s*(.+)', text_block)
    
    if not (cat_match or bold_pattern):
        issues.append("Entry missing recognized category tag or bold title header (e.g., '- **Provider Parity**: ...')")

    # Check for verification / witness pointer
    has_witness = bool(re.search(r'(verification|witness|mission\s*#|benchmark|passing|commit)', text_block, re.I))
    if not has_witness:
        issues.append("Entry missing witness/verification citation (e.g. 'Verification: Mission #402')")

    return {
        "valid": len(issues) == 0,
        "word_count": word_count,
        "issues": issues
    }

def analyze_mission_log(log_path: Path) -> Dict[str, Any]:
    """Scan mission log file and identify candidate wisdom signals and errors."""
    if not log_path.exists():
        return {"error": f"Log file not found: {log_path}", "signals": []}

    content = log_path.read_text(encoding="utf-8", errors="ignore")
    signals = []
    
    # Extract errors and exceptions
    error_lines = re.findall(r'(?:ERROR|FAIL|EXCEPTION|Traceback).*', content, re.I)
    # Extract resolved or pass indicators
    resolved_lines = re.findall(r'(?:RESOLVED|PASS|FIXED|SUCCESS).*', content, re.I)
    
    signals.append(f"Discovered {len(error_lines)} error signals and {len(resolved_lines)} recovery/pass signals.")
    
    return {
        "log_file": str(log_path),
        "total_lines": len(content.splitlines()),
        "signals": signals,
        "candidate_count": len(resolved_lines)
    }

def run_self_tests() -> bool:
    """Run built-in self-tests."""
    test_entry_valid = "- **Provider Parity**: Gemini models require 'google' as the provider key in TaskPayload. (Verification: Mission #402 pass)."
    res1 = validate_wisdom_entry(test_entry_valid)
    assert res1["valid"], f"Valid entry failed validation: {res1['issues']}"

    test_entry_short = "Too short."
    res2 = validate_wisdom_entry(test_entry_short)
    assert not res2["valid"], "Short entry should have failed"

    test_entry_no_witness = "- **SQLite Contention**: Enable WAL mode when 10+ agents are swarming to avoid timeouts."
    res3 = validate_wisdom_entry(test_entry_no_witness)
    assert not res3["valid"], "Entry lacking witness citation should fail"

    print("[PASS] debrief_analyzer self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Agent 99 Mission Debrief Analyzer")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    parser.add_argument("--validate-entry", type=str, help="Validate wisdom entry text")
    parser.add_argument("--scan-log", type=str, help="Path to mission log file to analyze")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.validate_entry:
        result = validate_wisdom_entry(args.validate_entry)
        if result["valid"]:
            print(f"[PASS] Wisdom entry compliant ({result['word_count']} words).")
            sys.exit(0)
        else:
            print(f"[FAIL] Wisdom entry non-compliant: {result['issues']}")
            sys.exit(1)

    if args.scan_log:
        result = analyze_mission_log(Path(args.scan_log))
        print(f"[OK] Log scan: {result}")
        sys.exit(0)

    parser.print_help()
    sys.exit(0)

if __name__ == "__main__":
    main()

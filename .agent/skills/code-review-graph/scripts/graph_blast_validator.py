#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Registry:Skills

### AI Context Alignment
- **Subsystem**: Agent Skills / code-review-graph / graph_blast_validator
- **Primary Entrypoints**: `parse_blast_context`, `verify_blast_radius_bounds`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic blast radius evaluation based on AST symbol graph connectivity.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[graph_blast_validator]`, `[GRAPH_BLAST]`, `[BLAST_RADIUS]`, `[PASS]`, `[FAIL]`
- **Witness Tests**: none declared
"""

import sys
import os
import json
import argparse
from pathlib import Path
from typing import List, Dict, Any, Optional, Union

# Windows console encoding fix
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

DEFAULT_MAX_CALLERS = 15

def parse_blast_context(data_or_path: Union[str, Path, Dict[str, Any]]) -> Dict[str, Any]:
    """Parse symbol graph audit context from dict, JSON string, or file path."""
    if isinstance(data_or_path, dict):
        return data_or_path

    if isinstance(data_or_path, (str, Path)):
        p = Path(data_or_path)
        if p.exists() and p.is_file():
            content = p.read_text(encoding='utf-8', errors='replace')
            return json.loads(content)
        elif isinstance(data_or_path, str) and (data_or_path.strip().startswith('{') or data_or_path.strip().startswith('[')):
            return json.loads(data_or_path)

    raise ValueError("Invalid blast context data or path provided.")

def verify_blast_radius_bounds(symbol_data: Dict[str, Any], max_callers: int = DEFAULT_MAX_CALLERS) -> Dict[str, Any]:
    """Verify that a symbol's caller count does not exceed safe blast radius thresholds."""
    callers = symbol_data.get("callers", [])
    caller_count = len(callers)
    symbol_name = symbol_data.get("node", {}).get("name", symbol_data.get("name", "unknown"))

    risk_level = "LOW"
    if caller_count > 10:
        risk_level = "HIGH"
    elif caller_count > 3:
        risk_level = "MEDIUM"

    exceeds_threshold = caller_count > max_callers

    return {
        "symbol": symbol_name,
        "caller_count": caller_count,
        "risk_level": risk_level,
        "max_callers_threshold": max_callers,
        "exceeds_threshold": exceeds_threshold,
        "callers": callers
    }

def run_self_tests() -> bool:
    """Execute built-in self-tests verifying blast radius evaluation."""
    print("🔬 [graph_blast_validator] Running self-test suite...")

    # Test 1: Low risk symbol
    low_risk = {
        "node": {"name": "calculate_hash"},
        "callers": [{"name": "signer"}, {"name": "verifier"}]
    }
    r1 = verify_blast_radius_bounds(low_risk, 15)
    assert r1["risk_level"] == "LOW" and not r1["exceeds_threshold"], f"Test 1 Failed: {r1}"

    # Test 2: High risk symbol
    high_risk = {
        "node": {"name": "AppState"},
        "callers": [{"name": f"caller_{i}"} for i in range(20)]
    }
    r2 = verify_blast_radius_bounds(high_risk, 15)
    assert r2["risk_level"] == "HIGH" and r2["exceeds_threshold"], f"Test 2 Failed: {r2}"

    # Test 3: Parse from JSON string
    raw_json = json.dumps(low_risk)
    parsed = parse_blast_context(raw_json)
    assert parsed["node"]["name"] == "calculate_hash", "Test 3 Failed: JSON string parsing"

    print("✅ [PASS] All self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Deterministic AST Graph Blast Radius Validator")
    parser.add_argument("--file", type=str, help="Path to audit_context.json or blast radius report")
    parser.add_argument("--max-callers", type=int, default=DEFAULT_MAX_CALLERS, help="Max caller threshold")
    parser.add_argument("--test", action="store_true", help="Run self-tests")
    parser.add_argument("--json", action="store_true", help="Output results in JSON format")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.file:
        target_path = Path(args.file)
    else:
        # Check default report if exists
        default_report = Path("reports/intelligence/audit_context.json")
        if default_report.exists():
            target_path = default_report
        else:
            print("ℹ️  No input file provided. Running self-test suite.")
            success = run_self_tests()
            sys.exit(0 if success else 1)

    print(f"🔍 [graph_blast_validator] Auditing AST Graph Blast Radius from: {target_path}")

    try:
        if str(target_path) == "stdin":
            data = parse_blast_context(sys.stdin.read())
        else:
            data = parse_blast_context(target_path)
    except Exception as e:
        print(f"❌ [FAIL] Failed to parse blast context: {e}", file=sys.stderr)
        sys.exit(1)

    top_connected = data.get("top_connected", [])
    if isinstance(data, list):
        top_connected = data
    elif not top_connected and "callers" in data:
        top_connected = [data]

    evaluations = [verify_blast_radius_bounds(s, args.max_callers) for s in top_connected]
    exceeded = [e for e in evaluations if e["exceeds_threshold"]]

    print(f"📊 [GRAPH_BLAST] Audited {len(evaluations)} connected symbol(s). High Blast Radius: {len(exceeded)}")

    if args.json:
        report = {
            "total_symbols": len(evaluations),
            "high_blast_symbols": len(exceeded),
            "evaluations": evaluations
        }
        print(json.dumps(report, indent=2))
        sys.exit(0 if len(exceeded) == 0 else 1)

    for e in evaluations[:5]:
        status_tag = "⚠️ [BLAST_RADIUS: HIGH]" if e["risk_level"] == "HIGH" else "ℹ️ [BLAST_RADIUS: OK]"
        print(f"  - {status_tag} '{e['symbol']}': {e['caller_count']} callers (Risk: {e['risk_level']})")

    if exceeded:
        print(f"\n⚠️  [BLAST_RADIUS] {len(exceeded)} symbol(s) exceed the threshold of {args.max_callers} callers.")
        print("    Mandatory Pre-Refactor Guard applies: inspect callers via `npm run graph:blast:guard`.")
        sys.exit(0) # Advisory exit
    else:
        print("\n✅ [PASS] All evaluated symbols fall within safe blast radius limits.")
        sys.exit(0)

if __name__ == "__main__":
    main()

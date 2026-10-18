#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / parallel-agents / orchestration_guard
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
import json
import argparse
from pathlib import Path
from typing import Dict, List, Any, Set

# Windows console encoding fix
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

def validate_subagent_briefing(briefing_text: str) -> Dict[str, Any]:
    """Validate that an agent briefing packet contains clear boundaries, file targets, and bounded tokens."""
    issues = []
    lines = [l.strip() for l in briefing_text.strip().splitlines() if l.strip()]
    if not lines:
        return {"valid": False, "issues": ["Empty briefing packet"]}

    words = len(" ".join(lines).split())
    if words > 1500:
        issues.append(f"Briefing payload too large ({words} words > 1500 limit). Risk of token bloat.")
    if words < 10:
        issues.append(f"Briefing packet too terse ({words} words). Missing task requirements.")

    # Check for target files or paths mentioned
    has_target = bool(re.search(r'(\b[a-zA-Z0-9_\-/\\]+\.(?:ts|tsx|rs|py|json|md)\b|`[^`]+`)', briefing_text))
    if not has_target:
        issues.append("Briefing does not specify concrete target files, symbols, or directories")

    # Check for expected deliverables or exit condition
    has_deliverables = bool(re.search(r'(deliverable|return|output|exit condition|verify|report)', briefing_text, re.I))
    if not has_deliverables:
        issues.append("Briefing lacks explicit expected deliverable or exit criteria")

    return {
        "valid": len(issues) == 0,
        "word_count": words,
        "issues": issues
    }

def detect_dependency_cycles(tasks: Dict[str, List[str]]) -> List[List[str]]:
    """Detect cycles in worker task dependency graphs."""
    cycles = []
    visited: Set[str] = set()
    rec_stack: Set[str] = set()
    path: List[str] = []

    def dfs(node: str):
        visited.add(node)
        rec_stack.add(node)
        path.append(node)

        for neighbor in tasks.get(node, []):
            if neighbor not in visited:
                dfs(neighbor)
            elif neighbor in rec_stack:
                cycle_start = path.index(neighbor)
                cycles.append(path[cycle_start:] + [neighbor])

        path.pop()
        rec_stack.remove(node)

    for task_name in tasks:
        if task_name not in visited:
            dfs(task_name)

    return cycles

def run_self_tests() -> bool:
    """Run built-in self-tests."""
    # Test valid briefing
    valid_briefing = """
    Target: server-rs/src/routes/system.rs
    Task: Audit concurrency locks in system status route.
    Deliverable: Return JSON report of discovered lock contentions and remediation diff.
    """
    res1 = validate_subagent_briefing(valid_briefing)
    assert res1["valid"], f"Valid briefing failed: {res1['issues']}"

    # Test cyclic graph detection
    cyclic_graph = {
        "task_a": ["task_b"],
        "task_b": ["task_c"],
        "task_c": ["task_a"]
    }
    cycles = detect_dependency_cycles(cyclic_graph)
    assert len(cycles) > 0, "Failed to detect cyclic task graph"

    # Test acyclic graph
    acyclic_graph = {
        "task_a": ["task_b", "task_c"],
        "task_b": ["task_d"],
        "task_c": ["task_d"],
        "task_d": []
    }
    cycles2 = detect_dependency_cycles(acyclic_graph)
    assert len(cycles2) == 0, "Acyclic graph should have 0 cycles"

    print("[PASS] orchestration_guard self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Multi-Agent Orchestration & Topology Guard")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    parser.add_argument("--check-briefing", type=str, help="Validate briefing packet text")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.check_briefing:
        res = validate_subagent_briefing(args.check_briefing)
        if res["valid"]:
            print(f"[PASS] Briefing valid ({res['word_count']} words).")
            sys.exit(0)
        else:
            print(f"[FAIL] Briefing non-compliant: {res['issues']}")
            sys.exit(1)

    parser.print_help()
    sys.exit(0)

if __name__ == "__main__":
    main()

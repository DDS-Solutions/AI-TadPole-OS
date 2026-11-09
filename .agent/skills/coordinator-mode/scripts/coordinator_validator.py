#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Registry:Skills

### AI Context Alignment
- **Subsystem**: Agent Skills / coordinator-mode / coordinator_validator
- **Primary Entrypoints**: `validate_worker_briefing`, `detect_concurrency_conflicts`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic validation of worker briefing prompts and concurrency dispatch safety.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[coordinator_validator]`, `[COORDINATOR]`, `[BRIEFING]`, `[CONFLICT]`, `[PASS]`, `[FAIL]`
- **Witness Tests**: none declared
"""

import sys
import os
import json
import re
import argparse
from pathlib import Path
from typing import List, Dict, Any, Optional

# Windows console encoding fix without swallowed exception
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

LAZY_DELEGATION_PATTERNS = [
    (r'\bfigure it out\b', "Lazy Delegation: Subagent prompt instructs worker to 'figure it out' without bounded scope"),
    (r'\bfix (?:whatever|all bugs?)\b', "Lazy Delegation: Open-ended instruction to fix unspecified bugs"),
    (r'\bdo whatever is needed\b', "Lazy Delegation: Unbounded directive lacking explicit task definition"),
    (r'\bbased on your findings,?\s*(?:go ahead and\s*)?fix\b', "Lazy Delegation: Violates non-negotiable directive against delegating understanding"),
]

def validate_worker_briefing(briefing_text: str) -> List[Dict[str, Any]]:
    """Scan worker briefing prompts for lazy delegation and missing constraints."""
    findings = []
    text_lower = briefing_text.lower()

    # 1. Check for lazy delegation phrases
    for pattern, desc in LAZY_DELEGATION_PATTERNS:
        match = re.search(pattern, text_lower)
        if match:
            item = {
                "category": "Lazy Delegation",
                "issue": desc,
                "matched_text": match.group(0)
            }
            findings.append(item)

    # 2. Check if implementation task lacks verification directive
    is_write_task = bool(re.search(r'(?:implement|modify|refactor|update|write)\b', text_lower))
    has_verification = bool(re.search(r'(?:verif|test|check|run\s+|prove)', text_lower))

    if is_write_task and not has_verification:
        item = {
            "category": "Missing Verification",
            "issue": "Implementation worker briefing lacks explicit verification instructions (test run or compilation check).",
            "matched_text": "Missing verification requirement"
        }
        findings.append(item)

    # 3. Check for missing bounded file target
    has_target = bool(re.search(r'(?:file|path|module|scope|lines?)\b', text_lower))
    if not has_target:
        item = {
            "category": "Unbounded Scope",
            "issue": "Worker briefing lacks specific target file, line, or module boundaries.",
            "matched_text": "Missing target boundaries"
        }
        findings.append(item)

    return findings

def detect_concurrency_conflicts(tasks: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    """Detect if multiple parallel tasks target the same file with write/edit operations."""
    conflicts = []
    write_targets: Dict[str, List[str]] = {}

    for t in tasks:
        task_id = t.get("id", "unknown_task")
        action = t.get("action", "").lower()
        target_files = t.get("target_files", [])

        if any(w in action for w in ["write", "edit", "modify", "refactor"]):
            for tf in target_files:
                tf_clean = str(tf).strip().replace('\\', '/').lower()
                if tf_clean not in write_targets:
                    write_targets[tf_clean] = []
                write_targets[tf_clean].append(task_id)

    for file_path, task_ids in write_targets.items():
        if len(task_ids) > 1:
            item = {
                "file": file_path,
                "conflicting_tasks": task_ids,
                "issue": f"File collision hazard: {len(task_ids)} concurrent write tasks targeting '{file_path}' without serialized locking."
            }
            conflicts.append(item)

    return conflicts

def run_self_tests() -> bool:
    """Execute built-in self-tests verifying worker briefing validation and conflict detection."""
    print("🔬 [coordinator_validator] Running self-test suite...")

    # Test 1: Lazy delegation detection
    lazy_prompt = "Task: Based on your findings, fix whatever is broken in the repo."
    f1 = validate_worker_briefing(lazy_prompt)
    assert any("Lazy Delegation" in f["category"] for f in f1), f"Test 1 Failed: Expected lazy delegation, got: {f1}"

    # Test 2: Compliant briefing pass
    good_prompt = """Role: Implementation Worker
Task: Modify src/routes/system.rs lines 40-55 to add telemetry tag.
Constraints: Do not touch AppState struct.
Verification: Run `cargo check` to confirm compilation.
"""
    f2 = validate_worker_briefing(good_prompt)
    assert len(f2) == 0, f"Test 2 Failed: Expected clean briefing, got: {f2}"

    # Test 3: Concurrency conflict detection
    parallel_tasks = [
        {"id": "worker_1", "action": "write", "target_files": ["src/routes/auth.rs"]},
        {"id": "worker_2", "action": "edit", "target_files": ["src/routes/auth.rs"]},
        {"id": "worker_3", "action": "read", "target_files": ["src/routes/auth.rs"]},
    ]
    conflicts = detect_concurrency_conflicts(parallel_tasks)
    assert len(conflicts) == 1 and len(conflicts[0]["conflicting_tasks"]) == 2, f"Test 3 Failed: {conflicts}"

    print("✅ [PASS] All self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Deterministic Multi-Agent Coordinator Validator")
    parser.add_argument("--file", type=str, help="Path to worker briefing prompt or dispatch plan JSON")
    parser.add_argument("--test", action="store_true", help="Run self-tests")
    parser.add_argument("--json", action="store_true", help="Output results in JSON format")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    content = ""
    target_name = "stdin"

    if args.file:
        file_path = Path(args.file)
        if not file_path.exists():
            print(f"❌ [FAIL] File not found: {file_path}", file=sys.stderr)
            sys.exit(1)
        target_name = str(file_path)
        content = file_path.read_text(encoding='utf-8', errors='replace')
    else:
        print("ℹ️  No input file provided. Running self-test suite.")
        success = run_self_tests()
        sys.exit(0 if success else 1)

    print(f"🔍 [coordinator_validator] Auditing multi-agent coordination from: {target_name}")

    # Check if input is dispatch plan JSON
    is_json = content.strip().startswith('{') or content.strip().startswith('[')
    if is_json:
        try:
            data = json.loads(content)
            tasks = data if isinstance(data, list) else data.get("tasks", [])
            conflicts = detect_concurrency_conflicts(tasks)
            print(f"📊 [COORDINATOR] Audited {len(tasks)} tasks. [CONFLICT] Detected: {len(conflicts)}")

            if args.json:
                print(json.dumps({"conflicts": conflicts, "passed": len(conflicts) == 0}, indent=2))
                sys.exit(0 if len(conflicts) == 0 else 1)

            if conflicts:
                print("❌ [FAIL] Concurrency conflicts detected:")
                for c in conflicts:
                    print(f"  ↳ [CONFLICT] {c['issue']}")
                sys.exit(1)
            else:
                print("✅ [PASS] Zero concurrent write conflicts detected.")
                sys.exit(0)
        except Exception:
            pass # Fall through to text analysis

    # Text worker briefing analysis
    findings = validate_worker_briefing(content)
    print(f"📡 [BRIEFING] Evaluated worker briefing prompt. Issues found: {len(findings)}")

    if args.json:
        print(json.dumps({"findings": findings, "passed": len(findings) == 0}, indent=2))
        sys.exit(0 if len(findings) == 0 else 1)

    if findings:
        print("❌ [FAIL] Worker briefing non-compliant with Coordinator standards:")
        for idx, f in enumerate(findings, 1):
            print(f"  {idx}. [{f['category']}] {f['issue']}")
        sys.exit(1)
    else:
        print("✅ [PASS] Worker briefing prompt adheres to Sovereign Conductor standards.")
        sys.exit(0)

if __name__ == "__main__":
    main()
